use std::fmt::Display;
use std::fs;
use std::future::Future;
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use clap::Parser;
use futures::future::Either;
use futures::lock::Mutex;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::args::PortPlumberArgs;
use crate::cmd_resource::CmdResource;
use crate::config::{PlumbingItemConfig, PortPlumberConfig};
use crate::connections_counter::ConnectionCounter;

mod config;
mod utils;
mod runner;
mod cmd_resource;
mod args;
mod connections_counter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args: PortPlumberArgs = PortPlumberArgs::parse();

    let config_file_path = if let Some(path) = args.config {
        path
    } else {
        config_from_user_dir()?
    };

    let config_content = fs::read_to_string(config_file_path)?;
    let config: PortPlumberConfig = toml::from_str(&config_content)?;

    futures::future::try_join_all(config.plumbing.into_iter().map(|(addr, conf)| listen_address(addr, conf))).await?;
    Ok(())
}

fn config_from_user_dir() -> anyhow::Result<PathBuf> {
    let Some(config_base_path) = dirs::config_dir() else {
        anyhow::bail!("Could not find os config dir")
    };
    let config_file_path = config_base_path.join("portplumber/config.toml");
    if !config_file_path.is_file() {
        anyhow::bail!("No config file was foud at {config_file_path:?}");
    } else {
        Ok(config_file_path)
    }
}

async fn listen_address(addr: impl ToSocketAddrs + Display, conf: PlumbingItemConfig) -> anyhow::Result<()> {
    log::info!("Starting listener for address {addr}");
    let listener = TcpListener::bind(addr).await?;

    let counter = Arc::new(Mutex::new(ConnectionCounter::new()));
    let mut resource = CmdResource::try_from(conf.resource.as_ref())?;

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
        let clouned_counter_mtx = counter.clone();
        tokio::spawn(async move {
            let res = redirect_stream(stream, conf.target).await;
            if let Err(err) = res {
                log::error!("Error processing stream - {err}");
            }
            let mut counter_guard = clouned_counter_mtx.lock().await;
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

async fn redirect_stream(incoming: TcpStream, addr: impl ToSocketAddrs) -> anyhow::Result<()> {
    let outgoing = TcpStream::connect(addr).await?;
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
        })?;
    Ok(())
}
