use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::args::PortPlumberArgs;
use crate::cmd_resource::CmdResource;
use crate::config::{PlumbingItemConfig, PortPlumberConfig};

mod config;
mod utils;
mod runner;
mod cmd_resource;
mod args;

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
