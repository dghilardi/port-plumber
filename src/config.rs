use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
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

#[derive(Deserialize, Clone)]
pub struct SocketConf<T> {
    pub sockets: BTreeMap<String, T>
}

#[derive(Deserialize)]
pub struct AddrPlumbingConfig {
    pub source: u16,
    pub target: SocketAddr,
    pub resource: Option<ResourceConfig>,
}

#[derive(Deserialize, Clone)]
pub struct NamePlumbingConfig {
    pub source: u16,
    pub target: u16,
    pub resource: ResourceConfig,
}

#[derive(Deserialize, Clone)]
pub struct ResourceConfig {
    #[serde(deserialize_with = "string_or_struct")]
    pub setup: CommandConfig,
    #[serde(default)]
    pub warmup_millis: u64,
}

#[derive(Deserialize, Clone)]
pub struct CommandConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "std::env::temp_dir")]
    pub workingdir: PathBuf,
}

impl CommandConfig {
    pub fn render_template<T: Serialize>(&self, data: &T) -> anyhow::Result<Self> {
        let h = Handlebars::new();
        Ok(Self {
            command: h.render_template(&self.command, data)?,
            args: self.args.iter()
                .map(|arg| h.render_template(arg, data))
                .collect::<Result<Vec<_>, _>>()?,
            workingdir: self.workingdir.to_str()
                .map(|workdir| h.render_template(workdir, data))
                .transpose()?
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir),
        })
    }
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