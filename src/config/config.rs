use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::network::network::NetworkConfig;
use crate::instance::instance::InstanceConfig;
use crate::interface::interface::InterfaceConfig;
use crate::route_table::route_table::RouteTableConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config{
    pub networks: HashMap<String, NetworkConfig>,
    pub instances: HashMap<String, InstanceConfig>,
    pub interfaces: HashMap<String, InterfaceConfig>,
    pub route_tables: HashMap<String, RouteTableConfig>,
}

impl Config{
    pub fn new() -> Config{
        Config{
            networks: HashMap::new(),
            instances: HashMap::new(),
            interfaces: HashMap::new(),
            route_tables: HashMap::new(),
        }
    }
}