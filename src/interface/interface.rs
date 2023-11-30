use std::collections::HashMap;
use std::net::Ipv4Addr;

use rand::Rng;
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
    pub routes: HashMap<String, InstanceInterface>,
}

impl InterfaceConfig{
    pub fn new(mtu: u32, network: &str, instance: &str, routes: HashMap<String, InstanceInterface>) -> InterfaceConfig{
        InterfaceConfig{
            mtu,
            network: network.to_string(),
            instance: instance.to_string(),
            routes,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceInterface{
    pub instance: String,
    pub interface: String,
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
    pub prefix_len: Option<u8>,
    pub managed: Option<String>,
    pub mac_address: Option<String>,
    pub routes: HashMap<ipnet::Ipv4Net, Ipv4Addr>,
}

fn generate_mac_address() -> String {
    let mut rng = rand::thread_rng();
    let mut mac = String::new();
    for i in 0..6 {
        let mut number = rng.gen_range(0..255);
        if i == 0 {
            number = match unset_bit(number, 0) {
                Ok(val) => val,
                Err(e) => panic!("{}", e),
            };
        }
        if i != 0 {
            mac.push(':');
        }
        mac.push_str(&format!("{:02X}", number));
    }
    mac.to_lowercase()
}

fn unset_bit(b: u8, bit_number: i32) -> Result<u8, &'static str> {
    if bit_number < 8 && bit_number > -1 {
        Ok(b & !(0x01 << bit_number))
    } else {
        Err("BitNumber was not in the valid range! (BitNumber = (min)0 - (max)7)")
    }
}

impl InterfaceRuntime{
    pub fn configure_routes(config: &Config, networks: &mut HashMap<String, NetworkRuntime>, instances: &mut HashMap<String, InstanceRuntime>){
        for (name, interface) in &config.interfaces{
            let network = networks.get_mut(&interface.network).unwrap();
            let instances_clone = instances.clone();
            let instance = instances.get_mut(&interface.instance).unwrap();
            match &network.network_type{
                NetworkTypeRuntime::Unmanaged{subnet: _, addresses: _, gateway: _} => {
                    let mut interface_routes = HashMap::new();
                    for (dst_network, instance_interface) in &interface.routes{
                        let dst_network = networks.get(dst_network).unwrap();
                        let nh_instance = if let Some(nh_instance) = instances_clone.get(&instance_interface.instance){
                            nh_instance
                        } else {
                            continue;
                        };
                        let nh_interface = if let Some(nh_interface) = nh_instance.interfaces.get(&instance_interface.interface){
                            nh_interface
                        } else {
                            continue;
                        };                        
                        match &dst_network.network_type{
                            NetworkTypeRuntime::Unmanaged { subnet, addresses: _, gateway: _ } =>{
                                interface_routes.insert(subnet.clone(), nh_interface.address.unwrap());
                            },
                            _=>{},
                        }
                    }
                    if let Some(interface) = instance.interfaces.get_mut(name){
                        interface.routes = interface_routes;
                    }
                },
                _ => {}
            }
        }
    }
    pub fn configure_addresses(config: &Config, networks: &mut HashMap<String, NetworkRuntime>, instances: &mut HashMap<String, InstanceRuntime>){
        for (name, interface) in &config.interfaces{
            let network = networks.get_mut(&interface.network).unwrap();
            let instance = instances.get_mut(&interface.instance).unwrap();
            match &network.network_type{
                NetworkTypeRuntime::Unmanaged{subnet: _, addresses: _, gateway: _} => {
                    let (address, prefix_len) = network.assign_address().unwrap();
                    let mtu = interface.mtu;
                    let interface = InterfaceRuntime{
                        mtu,
                        address: Some(address),
                        prefix_len: Some(prefix_len),
                        mac_address: Some(generate_mac_address()),
                        routes: HashMap::new(),
                        managed: None,
                    };
                    instance.interfaces.insert(name.clone(), interface);
                },
                NetworkTypeRuntime::Managed{name} => {
                    let mtu = interface.mtu;
                    let interface = InterfaceRuntime{
                        mtu,
                        address: None,
                        prefix_len: None,
                        mac_address: Some(generate_mac_address()),
                        routes: HashMap::new(),
                        managed: Some(name.clone()),
                    };
                    instance.interfaces.insert(name.clone(), interface);
                },
            }
        }
    }
}