use std::collections::BTreeMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use serde::Deserialize;
use crate::utils::serde::string_or_struct;

#[derive(Deserialize)]
pub struct PortPlumberConfig {
    pub plumbing: BTreeMap<SocketAddr, PlumbingItemConfig>
}

#[derive(Deserialize)]
pub struct PlumbingItemConfig {
    pub target: SocketAddr,
    pub resource: Option<ResourceConfig>,
}

#[derive(Deserialize)]
pub struct ResourceConfig {
    #[serde(deserialize_with = "string_or_struct")]
    pub setup: CommandConfig,
}

#[derive(Deserialize)]
pub struct CommandConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

impl FromStr for CommandConfig {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            command: String::from(s),
            args: Vec::new(),
        })
    }
}