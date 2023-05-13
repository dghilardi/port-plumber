use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use crate::config::ResourceConfig;

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
    resource: Option<ResourceConfig>,
}

impl Plumber {
    pub fn new() -> Self {
        Self {
            in_range: Arc::new(Mutex::new(IpAddr::from([127, 127, 0, 0]))),
            out_range: Arc::new(Mutex::new(IpAddr::from([127, 191, 0, 0]))),
            plumbing: Default::default(),
        }
    }

    pub fn resolve(&self, name: &str) -> IpAddr {
        let entry = self.plumbing
            .entry(String::from(name))
            .or_insert_with(|| Plumbing {
                in_addr: self.in_range.lock().expect("Broken in_range mutex").increment(),
                out_addr: self.out_range.lock().expect("Broken out_range mutex").increment(),
                sockets: Vec::new(),
            });

        entry.in_addr
    }
}

trait Increment: Sized {
    fn increment(&mut self) -> Self;
}

impl Increment for IpAddr {
    fn increment(&mut self) -> Self {
        match self {
            IpAddr::V4(ipv4) => {
                let value = u32::from(*ipv4) + 1;
                *self = Self::from(Ipv4Addr::from(value));
                self.clone()
            },
            IpAddr::V6(_) => unimplemented!(),
        }
    }
}