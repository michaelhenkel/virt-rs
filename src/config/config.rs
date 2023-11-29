use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::network::network::NetworkConfig;
use crate::instance::instance::InstanceConfig;
use crate::interface::interface::InterfaceConfig;
use crate::route_table::route_table::RouteTableConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config{
    pub user_config: Option<UserConfig>,
    pub networks: HashMap<String, NetworkConfig>,
    pub instances: HashMap<String, InstanceConfig>,
    pub interfaces: HashMap<String, InterfaceConfig>,
    pub route_tables: HashMap<String, RouteTableConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserConfig{
    pub user_name: String,
    pub key_path: String,
}

impl Config{
    pub fn new(user_config: Option<UserConfig>) -> Config{
        Config{
            user_config,
            networks: HashMap::new(),
            instances: HashMap::new(),
            interfaces: HashMap::new(),
            route_tables: HashMap::new(),
        }
    }
}