use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs;
use std::future::Future;
use std::net::IpAddr;
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use anyhow::Context;

use clap::Parser;
use futures::future::Either;
use futures::lock::Mutex;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use crate::api::build_server;

use crate::args::PortPlumberArgs;
use crate::cmd_resource::CmdResource;
use crate::config::{NamePlumbingConfig, PlumbingItemConfig, PortPlumberConfig, SocketConf};
use crate::connections_counter::ConnectionCounter;
use crate::plumber::{Plumber, PlumbingDescriptor};
use crate::resolver::NameResolver;

mod config;
mod utils;
mod runner;
mod cmd_resource;
mod args;
mod connections_counter;
mod api;
mod plumber;
mod ext;
mod resolver;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args: PortPlumberArgs = PortPlumberArgs::parse();

    let config_file_path = if let Some(path) = args.config {
        path
    } else {
        config_from_user_dir()?
    };

    let config_content = fs::read_to_string(config_file_path)
        .context("Error loading config file")?;
    let config: PortPlumberConfig = toml::from_str(&config_content)
        .context("Error parsing config file")?;


    let plumber = Plumber::new();
    let mut resolv_conf: BTreeMap<String, SocketConf<NamePlumbingConfig>> = BTreeMap::new();

    for (name, plumbing) in config.plumbing {
        match plumbing {
            PlumbingItemConfig::Addr(conf) => {
                let in_addr: IpAddr = name.parse()?;
                for (_name, socket) in conf.sockets {
                    plumber.attach(&name, PlumbingDescriptor {
                        in_addr: Some(in_addr),
                        in_port: socket.source,
                        out_addr: Some(socket.target.ip()),
                        out_port: socket.target.port(),
                        resource: socket.resource,
                    })?;
                }
            }
            PlumbingItemConfig::Name(conf) => {
                resolv_conf.insert(name, conf);
            },
        }
    }

    let name_resolver = NameResolver::new(resolv_conf, plumber.clone());

    let cmd_path = std::env::var("CMD_SOCKET")
        .ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or(config.socket);

    if let Some(ref socket) =  cmd_path {
        log::debug!("Starting socket server");
        let server = build_server(socket, name_resolver)
            .context("Error building server")?;
        tokio::spawn(async move {
            let out = server.await;
            if let Err(err) = out {
                log::error!("Error during socket server execution - {err}");
            }
        });
    }

    //futures::future::try_join_all(config.plumbing.into_iter().map(|(addr, conf)| listen_address(addr, conf))).await?;
    plumber.join().await?;
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
