use std::fmt::{Debug, Display};
use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use anyhow::{anyhow, Context};

use dashmap::DashMap;
use dashmap::mapref::multiple::RefMulti;
use futures::future::Either;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::task::JoinHandle;

use crate::cmd_resource::CmdResource;
use crate::config::{PlumbingItemConfig, ResourceConfig};
use crate::connections_counter::ConnectionCounter;
use crate::ext::addr::Increment;

#[derive(Clone)]
pub struct Plumber {
    in_range: Arc<Mutex<IpAddr>>,
    out_range: Arc<Mutex<IpAddr>>,
    plumbing: Arc<DashMap<String, Plumbing>>
}

struct Plumbing {
    in_addr: IpAddr,
    out_addr: IpAddr,
    sockets: Vec<MappedSocket>,
}
struct MappedSocket {
    in_port: u16,
    out_port: u16,
    handle: JoinHandle<()>,
}

pub struct AddressBinding {
    pub source: IpAddr,
    pub target: IpAddr
}

#[derive(Debug)]
pub struct PlumbingDescriptor {
    pub in_addr: Option<IpAddr>,
    pub in_port: u16,
    pub out_addr: Option<IpAddr>,
    pub out_port: u16,
    pub resource: Option<ResourceConfig>,
}

impl Plumber {
    pub fn new() -> Self {
        Self {
            in_range: Arc::new(Mutex::new(IpAddr::from([127, 127, 0, 0]))),
            out_range: Arc::new(Mutex::new(IpAddr::from([127, 191, 0, 0]))),
            plumbing: Default::default(),
        }
    }

    pub fn resolve(&self, name: &str) -> AddressBinding {
        let entry = self.resolve_plumbing(name, None, None);
        AddressBinding {
            source: entry.in_addr,
            target: entry.out_addr,
        }
    }

    fn resolve_plumbing(&self, name: &str, in_addr: Option<IpAddr>, out_addr: Option<IpAddr>) -> dashmap::mapref::one::RefMut<String, Plumbing> {
        self.plumbing
            .entry(String::from(name))
            .or_insert_with(|| Plumbing {
                in_addr: in_addr.unwrap_or_else(|| self.in_range.lock().expect("Broken in_range mutex").increment()),
                out_addr: out_addr.unwrap_or_else(|| self.out_range.lock().expect("Broken out_range mutex").increment()),
                sockets: Vec::new(),
            })
    }


    pub fn attach(&self, name: &str, descriptor: PlumbingDescriptor) -> anyhow::Result<()> {
        log::debug!("attach: {descriptor:?}");
        let mut entry = self.resolve_plumbing(name, descriptor.in_addr, descriptor.out_addr);
        log::debug!("entry: {} -> {}", entry.in_addr, entry.out_addr);
        if let Some(plumbing) = entry.sockets.iter().find(|s| s.in_port == descriptor.in_port) {
            log::debug!("Plumbing already defined for {}:{} to {}:{}", entry.in_addr, plumbing.in_port, entry.out_addr, plumbing.out_port)
        } else {
            let source_socket = SocketAddr::new(entry.in_addr, descriptor.in_port);

            let out_addr = descriptor.out_addr.unwrap_or(entry.out_addr);
            let target_socket = SocketAddr::new(out_addr, descriptor.out_port);

            log::debug!("{}:{} -> {}:{}", entry.value().in_addr, descriptor.in_port, out_addr, descriptor.out_port);

            let handle = tokio::spawn(async move {
                let out = listen_address(source_socket, target_socket, descriptor.resource).await;
                if let Err(err) = out {
                    log::error!("Error listening address {source_socket}")
                }
            });
            entry.sockets.push(MappedSocket {
                in_port: descriptor.in_port,
                out_port: descriptor.out_port,
                handle,
            })
        }
        Ok(())
    }

    pub async fn join(self) -> anyhow::Result<()> {
        while !self.plumbing.is_empty() {
            let key = {
                let entry = self.plumbing.iter().next().ok_or_else(|| anyhow!("Could not find any key in plumbing"))?;
                entry.key().to_string()
            };

            let (key, plumbing) = self.plumbing.remove(&key)
                .ok_or_else(|| anyhow!("Could not find plumbing for entry {}", key))?;
            for socket in plumbing.sockets {
                let out = socket.handle.await;
                if let Err(err) = out {
                    log::error!("Join error - {err}");
                }
            }
            log::debug!("{key} plumbing terminated");
        }
        Ok(())
    }
}

async fn listen_address(source: SocketAddr, target:SocketAddr, resource: Option<ResourceConfig>) -> anyhow::Result<()> {
    log::info!("Starting listener for address {source}");
    let listener = TcpListener::bind(source).await?;

    let counter = Arc::new(tokio::sync::Mutex::new(ConnectionCounter::new()));
    let mut resource = CmdResource::try_from(resource.as_ref())?;

    loop {
        let Some((stream, _)) = timeout(Duration::from_secs(30), listener.accept()).await? else {
            if let Some(ts) = counter.lock().await.no_connections_since() {
                if ts.add(Duration::from_secs(600)) < SystemTime::now() {
                    resource.ensure_stopped()?;
                }
            }
            continue;
        };
        {
            let mut counter_guard = counter.lock().await;
            counter_guard.add_connection();
        }
        resource.ensure_running().await?;
        let cloned_counter_mtx = counter.clone();
        tokio::spawn(async move {
            log::debug!("SOURCE: {source} TARGET: {target}");
            let res = redirect_stream(stream, target).await;
            if let Err(err) = res {
                log::error!("Error processing stream - {err}");
            }
            let mut counter_guard = cloned_counter_mtx.lock().await;
            counter_guard.rem_connection();
        });
    }
}

async fn timeout<F, O, E>(duration: Duration, future: F) -> Result<Option<O>, E>
    where
        F: Future<Output=Result<O, E>>,
{
    tokio::time::timeout(duration, future).await.ok().transpose()
}

async fn redirect_stream(incoming: TcpStream, addr: impl ToSocketAddrs + Copy + Debug) -> anyhow::Result<()> {
    let outgoing = TcpStream::connect(addr).await
        .with_context(|| format!("Error connecting to address {addr:?}"))?;

    let (mut in_reader, mut in_writer) = incoming.into_split();
    let (mut out_reader, mut out_writer) = outgoing.into_split();

    futures::future::try_select(
        Box::pin(tokio::io::copy(&mut in_reader, &mut out_writer)),
        Box::pin(tokio::io::copy(&mut out_reader, &mut in_writer)),
    )
        .await
        .map_err(|e| match e {
            Either::Left((err, _fut)) => err,
            Either::Right((err, _fut)) => err,
        })
        .context("Error during socket copy")?;
    Ok(())
}