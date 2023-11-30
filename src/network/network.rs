use std::collections::{BTreeMap, HashMap};
use std::net::Ipv4Addr;
use serde::{Deserialize, Serialize};
use crate::object::object::Object;
use crate::config::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig{
    pub network_type: NetworkTypeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum NetworkTypeConfig{
    Managed{
        name: String,
    },
    Unmanaged{
        subnet: String,
    },
}

impl NetworkConfig{
    pub fn new(network_type: NetworkTypeConfig) -> NetworkConfig{
        NetworkConfig{
            network_type,
        }
    }
}

impl <'a>Object<'a, NetworkConfig> for Config {
    fn get(&'a self, name: &str) -> Option<&'a NetworkConfig> {
        self.networks.get(name)
    }
    fn get_mut(&'a mut self, name: &str) -> Option<&'a mut NetworkConfig> {
        self.networks.get_mut(name)
    }
    fn add(&mut self, name: &str, value: NetworkConfig) {
        self.networks.insert(name.to_string(), value);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkRuntime{
    pub network_type: NetworkTypeRuntime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum NetworkTypeRuntime{
    Managed{
        name: String,
    },
    Unmanaged{
        subnet: ipnet::Ipv4Net,
        addresses: BTreeMap<u32, Ipv4Addr>,
        gateway: Ipv4Addr,
    },
}

impl NetworkRuntime{
    pub fn assign_address(&mut self) -> Option<(Ipv4Addr,u8)>{
        match self.network_type{
            NetworkTypeRuntime::Unmanaged{subnet, ref mut addresses, gateway: _} => {
                let mut first_address = u32::from_be_bytes(subnet.network().octets());
                first_address += 1;
                loop {
                    if !addresses.contains_key(&first_address){
                        let address = Ipv4Addr::from(first_address);
                        let prefix_len = subnet.prefix_len();
                        addresses.insert(first_address, address);
                        return Some((address, prefix_len));
                    }
                    first_address += 1;
                }
            },
            _ => None,
        }

    }
}

impl From<NetworkConfig> for NetworkRuntime{
    fn from(config: NetworkConfig) -> Self {
        match config.network_type{
            NetworkTypeConfig::Unmanaged { subnet } => {
                let subnet: ipnet::Ipv4Net = subnet.parse().unwrap();
                let first_address = u32::from_be_bytes(subnet.network().octets()) + 1;
                let gateway = Ipv4Addr::from(first_address);
                let mut addresses = BTreeMap::new();
                addresses.insert(first_address, gateway);
                NetworkRuntime{
                    network_type: NetworkTypeRuntime::Unmanaged{
                        subnet,
                        addresses,
                        gateway,
                    }
                }
            },
            NetworkTypeConfig::Managed { name } => {
                NetworkRuntime{
                    network_type: NetworkTypeRuntime::Managed{
                        name,
                    }
                }
            }
        }


    }
}

impl From<&Config> for HashMap<String,NetworkRuntime>{
    fn from(config: &Config) -> Self {
        let mut networks = HashMap::new();
        for (name, network) in &config.networks{
            networks.insert(name.to_string(), NetworkRuntime::from(network.clone()));
        }
        networks
    }
}

