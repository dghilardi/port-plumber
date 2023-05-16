use std::collections::BTreeMap;
use std::net::IpAddr;
use crate::config::{NamePlumbingConfig, SocketConf};
use crate::plumber::Plumber;

#[derive(Clone)]
pub struct NameResolver {
    config: BTreeMap<String, SocketConf<NamePlumbingConfig>>,
    plumber: Plumber,
}

impl NameResolver {
    pub fn new(
        config: BTreeMap<String, SocketConf<NamePlumbingConfig>>,
        plumber: Plumber,
    ) -> Self {
        Self { config, plumber }
    }

    pub fn resolve(&self, name: &str) -> Option<IpAddr> {
        let conf_name = self.config.iter()
            .find(|(entry_name, _)| name.ends_with(*entry_name));

        let Some((matched_name, socket_conf)) = conf_name else {
            return None;
        };

        for conf in &socket_conf.sockets {

        }

        let ip = self.plumber.resolve(name);
        Some(ip)
    }
}