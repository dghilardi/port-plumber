mod config;
mod utils;
mod runner;
mod cmd_resource;

use std::fs;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use crate::cmd_resource::CmdResource;
use crate::config::{PlumbingItemConfig, PortPlumberConfig};
use crate::runner::CmdRunner;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let config_content = fs::read_to_string("config/portplumber.toml")?;
    let config: PortPlumberConfig = toml::from_str(&config_content)?;

    futures::future::try_join_all(config.plumbing.into_iter().map(|(addr, conf)| listen_address(addr, conf))).await.expect("Error listening traffic");
    Ok(())
}

async fn listen_address(addr: impl ToSocketAddrs, conf: PlumbingItemConfig) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut resource = CmdResource::try_from(conf.resource.as_ref())?;
    loop {
        let (stream, _socket) = listener.accept().await?;
        resource.ensure_running().await?;
        tokio::spawn(async move {
            let res = redirect_stream(stream, conf.target).await;
        });
    }
}

async fn redirect_stream(incoming: TcpStream, addr: impl ToSocketAddrs) -> anyhow::Result<()> {
    let outgoing = TcpStream::connect(addr).await?;
    let (mut in_reader, mut in_writer) = incoming.into_split();
    let (mut out_reader, mut out_writer) = outgoing.into_split();
    futures::future::try_join(
    tokio::io::copy(&mut in_reader, &mut out_writer),
    tokio::io::copy(&mut out_reader, &mut in_writer),
    ).await?;
    Ok(())
}
