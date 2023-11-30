use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{instance::instance::InstanceRuntime, network::network::NetworkRuntime, config::config::{Config, UserConfig}, interface::interface::InterfaceRuntime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Runtime{
    pub user_config: UserConfig,
    pub instances: HashMap<String, InstanceRuntime>,
    pub networks: HashMap<String, NetworkRuntime>,
    pub mac_table: Option<MacTable>,
}

impl Runtime{
    pub fn build(config: &Config) -> Runtime{
        let mut networks: HashMap<String, NetworkRuntime> = HashMap::from(config);
        let mut instances: HashMap<String, InstanceRuntime> = HashMap::from(config);
        InterfaceRuntime::configure_addresses(config, &mut networks, &mut instances);
        InterfaceRuntime::configure_routes(config, &mut networks, &mut instances);
        Runtime{
            user_config: config.user_config.clone(),
            instances,
            networks,
            mac_table: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MacTable{
}