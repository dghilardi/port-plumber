use std::collections::{BTreeMap, HashMap};
use std::net::IpAddr;
use crate::config::{NamePlumbingConfig, ResourceConfig, SocketConf};
use crate::plumber::{Plumber, PlumbingDescriptor};
use serde::Serialize;

#[derive(Clone)]
pub struct NameResolver {
    config: BTreeMap<String, SocketConf<NamePlumbingConfig>>,
    plumber: Plumber,
}

#[derive(Serialize)]
pub struct TemplateParams {
    source: EndpointParam,
    target: EndpointParam,
    url: UrlParam,
}

#[derive(Serialize)]
pub struct UrlParam {
    full: String,
    parts: HashMap<usize, String>,
}

#[derive(Serialize)]
pub struct EndpointParam {
    ip: IpAddr,
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

        let binding = self.plumber.resolve(name);
        for (_, conf) in &socket_conf.sockets {
            let setup = match conf.resource.setup.render_template(&TemplateParams {
                source: EndpointParam { ip: binding.source },
                target: EndpointParam { ip: binding.target },
                url: UrlParam {
                    full: String::from(name),
                    parts: name.split('.')
                        .rev()
                        .enumerate()
                        .map(|(idx, s)| (idx, String::from(s)))
                        .collect(),
                },
            }) {
                Ok(setup) => setup,
                Err(err) => {
                    log::error!("Error rendering configuration template - {err}");
                    continue
                }
            };
            let out = self.plumber.attach(name, PlumbingDescriptor {
                in_addr: None,
                in_port: conf.source,
                out_addr: None,
                out_port: conf.target,
                resource: Some(ResourceConfig {
                    warmup_millis: conf.resource.warmup_millis,
                    setup,
                }),
            });
            if let Err(err) = out {
                log::error!("Error binding address - {err}");
            }
        }

        Some(binding.source)
    }
}