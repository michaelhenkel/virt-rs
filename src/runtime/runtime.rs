use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{instance::instance::InstanceRuntime, network::network::NetworkRuntime, config::config::{Config, UserConfig}, interface::interface::InterfaceRuntime, route_table::route_table::RouteTableRuntime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Runtime{
    pub user_config: Option<UserConfig>,
    pub instances: HashMap<String, InstanceRuntime>,
    pub networks: HashMap<String, NetworkRuntime>,
}

impl Runtime{
    pub fn build(config: &Config) -> Runtime{
        let mut networks: HashMap<String, NetworkRuntime> = HashMap::from(config);
        let mut instances: HashMap<String, InstanceRuntime> = HashMap::from(config);
        InterfaceRuntime::configure(config, &mut networks, &mut instances);
        RouteTableRuntime::configure(config, &networks, &mut instances);
        Runtime{
            user_config: config.user_config.clone(),
            instances,
            networks,
        }
    }
}