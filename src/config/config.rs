use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::network::network::NetworkConfig;
use crate::instance::instance::InstanceConfig;
use crate::interface::interface::InterfaceConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config{
    pub user_config: UserConfig,
    pub networks: HashMap<String, NetworkConfig>,
    pub instances: HashMap<String, InstanceConfig>,
    pub interfaces: HashMap<String, InterfaceConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserConfig{
    pub user_name: String,
    pub key_path: String,
    pub base_directory: String,
}

impl Config{
    pub fn new(user_config: UserConfig) -> Config{
        Config{
            user_config,
            networks: HashMap::new(),
            instances: HashMap::new(),
            interfaces: HashMap::new(),
        }
    }
}