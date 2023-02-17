use std::collections::BTreeMap;
use std::net::SocketAddr;
use serde::Deserialize;

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
    setup: Vec<String>,
}