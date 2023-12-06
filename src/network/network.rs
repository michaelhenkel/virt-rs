use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::sync::{Mutex, Arc};
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
        match network_type{
            NetworkTypeConfig::Unmanaged {subnet} => {
                NetworkConfig{
                    network_type: NetworkTypeConfig::Unmanaged{
                        subnet,
                    }
                }
            },
            NetworkTypeConfig::Managed {name } => {
                NetworkConfig{
                    network_type: NetworkTypeConfig::Managed{
                        name,
                    }
                }
            },
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
    fn add(&mut self, name: &str,  value: NetworkConfig) {
        self.networks.insert(name.to_string(), value);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkRuntime{
    pub network_type: NetworkTypeRuntime,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum NetworkTypeRuntime{
    Managed{
        name: String,
    },
    Unmanaged{
        subnet: String,
        assigned_addresses: Option<BTreeMap<u32, Ipv4Addr>>,
        gateway: Option<Ipv4Addr>,
    },
}

impl NetworkRuntime{
    pub fn new(network_config: &NetworkConfig) -> Arc<Mutex<NetworkRuntime>>{
        match &network_config.network_type{
            NetworkTypeConfig::Unmanaged {subnet} => {
                let subnet: ipnet::Ipv4Net = subnet.parse().unwrap();
                let first_address = u32::from_be_bytes(subnet.network().octets()) + 1;
                let gateway = Ipv4Addr::from(first_address);
                Arc::new(Mutex::new(NetworkRuntime{
                    network_type: NetworkTypeRuntime::Unmanaged{
                        subnet: subnet.to_string(),
                        assigned_addresses: Some(BTreeMap::from([(
                            first_address,
                            gateway
                        )])),
                        gateway: Some(gateway),
                    }
                }))
            },
            NetworkTypeConfig::Managed {name } => {
                Arc::new(Mutex::new(NetworkRuntime{
                    network_type: NetworkTypeRuntime::Managed{
                        name: name.to_string(),
                    }
                }))
            },
        }
    }
    pub fn assign_address(&mut self) -> Option<(Ipv4Addr,u8)>{
        match &mut self.network_type{
            NetworkTypeRuntime::Unmanaged{subnet, ref mut assigned_addresses, gateway: _} => {
                let subnet: ipnet::Ipv4Net = subnet.parse().unwrap();
                let addresses = assigned_addresses.as_mut().unwrap();
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