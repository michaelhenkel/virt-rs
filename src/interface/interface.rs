use std::collections::HashMap;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use crate::instance::instance::InstanceRuntime;
use crate::object::object::Object;
use crate::config::config::Config;
use crate::network::network::{NetworkRuntime, NetworkTypeRuntime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InterfaceConfig{
    pub mtu: u32,
    pub network: String,
    pub instance: String,
}

impl InterfaceConfig{
    pub fn new(mtu: u32, network: &str, instance: &str) -> InterfaceConfig{
        InterfaceConfig{
            mtu,
            network: network.to_string(),
            instance: instance.to_string(),
        }
    }
}

impl <'a>Object<'a, InterfaceConfig> for Config {
    fn get(&'a self, name: &str) -> Option<&'a InterfaceConfig> {
        self.interfaces.get(name)
    }
    fn get_mut(&'a mut self, name: &str) -> Option<&'a mut InterfaceConfig> {
        self.interfaces.get_mut(name)
    }
    fn add(&mut self, name: &str, value: InterfaceConfig) {
        self.interfaces.insert(name.to_string(), value);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InterfaceRuntime{
    pub mtu: u32,
    pub address: Option<Ipv4Addr>,
    pub managed: Option<String>,
}

impl InterfaceRuntime{
    pub fn configure(config: &Config, networks: &mut HashMap<String, NetworkRuntime>, instances: &mut HashMap<String, InstanceRuntime>){
        for (name, interface) in &config.interfaces{
            let network = networks.get_mut(&interface.network).unwrap();
            let instance = instances.get_mut(&interface.instance).unwrap();
            match &network.network_type{
                NetworkTypeRuntime::Unmanaged{subnet: _, addresses: _, gateway: _} => {
                    let address = network.assign_address().unwrap();
                    let mtu = interface.mtu;
                    let interface = InterfaceRuntime{
                        mtu,
                        address: Some(address),
                        managed: None,
                    };
                    instance.interfaces.insert(name.clone(), interface);
                },
                NetworkTypeRuntime::Managed{name} => {
                    let mtu = interface.mtu;
                    let interface = InterfaceRuntime{
                        mtu,
                        address: None,
                        managed: Some(name.clone()),
                    };
                    instance.interfaces.insert(name.clone(), interface);
                },
            }
        }
    }
}