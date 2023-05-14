use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use serde::Deserialize;
use crate::utils::serde::string_or_struct;

#[derive(Deserialize)]
pub struct PortPlumberConfig {
    pub socket: Option<PathBuf>,
    pub plumbing: BTreeMap<String, PlumbingItemConfig>,
}

#[derive(Deserialize)]
#[serde(tag = "mode")]
pub enum PlumbingItemConfig {
    Addr(SocketConf<AddrPlumbingConfig>),
    Name(SocketConf<NamePlumbingConfig>),
}

#[derive(Deserialize)]
pub struct SocketConf<T> {
    pub sockets: BTreeMap<String, T>
}

#[derive(Deserialize)]
pub struct AddrPlumbingConfig {
    pub source: u16,
    pub target: SocketAddr,
    pub resource: Option<ResourceConfig>,
}

#[derive(Deserialize)]
pub struct NamePlumbingConfig {
    pub source: u16,
    pub target: u16,
    pub resource: ResourceConfig,
}

#[derive(Deserialize)]
pub struct ResourceConfig {
    #[serde(deserialize_with = "string_or_struct")]
    pub setup: CommandConfig,
    #[serde(default)]
    pub warmup_millis: u64,
}

#[derive(Deserialize)]
pub struct CommandConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "std::env::temp_dir")]
    pub workingdir: PathBuf,
}

impl FromStr for CommandConfig {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            command: String::from(s),
            args: Vec::new(),
            workingdir: std::env::temp_dir(),
        })
    }
}