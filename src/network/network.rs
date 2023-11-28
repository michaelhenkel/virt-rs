use std::collections::{BTreeMap, HashMap};
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use crate::object::object::Object;
use crate::config::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig{
    pub subnet: String,
}

impl NetworkConfig{
    pub fn new(subnet: &str) -> NetworkConfig{
        NetworkConfig{
            subnet: subnet.to_string(),
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
    pub subnet: ipnet::Ipv4Net,
    pub addresses: BTreeMap<u32, Ipv4Addr>,
    pub gateway: Ipv4Addr,
}

impl NetworkRuntime{
    pub fn assign_address(&mut self) -> Ipv4Addr{
        let mut first_address = u32::from_be_bytes(self.subnet.network().octets());
        first_address += 1;
        loop {
            if !self.addresses.contains_key(&first_address){
                let address = Ipv4Addr::from(first_address);
                self.addresses.insert(first_address, address);
                return address;
            }
            first_address += 1;
        }
    }
}

impl From<NetworkConfig> for NetworkRuntime{
    fn from(config: NetworkConfig) -> Self {
        let subnet: ipnet::Ipv4Net = config.subnet.parse().unwrap();
        let first_address = u32::from_be_bytes(subnet.network().octets()) + 1;
        let gateway = Ipv4Addr::from(first_address);
        let mut addresses = BTreeMap::new();
        addresses.insert(first_address, gateway);
        NetworkRuntime{
            subnet,
            addresses,
            gateway,
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

