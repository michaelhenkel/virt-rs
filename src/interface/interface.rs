use crate::{Object, network::network::{NetworkRuntime, NetworkTypeRuntime}};
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::{network::network::NetworkConfig, instance::instance::InstanceConfig};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct InterfaceConfig{
    pub mtu: Option<u32>,
    pub network: String,
}

impl InterfaceConfig{
    pub fn new(mtu: Option<u32>, network: &str) -> InterfaceConfig{
        InterfaceConfig{
            mtu,
            network: network.to_string(),
        }
    }
}

impl <'a>Object<'a, InterfaceConfig> for InstanceConfig {
    fn get(&'a self, name: &str) -> Option<&'a InterfaceConfig> {
        self.interfaces.get(name)
    }
    fn get_mut(&'a mut self, name: &str) -> Option<&'a mut InterfaceConfig> {
        self.interfaces.get_mut(name)
    }
    fn add(&mut self, name: &str,  value: InterfaceConfig) {
        self.interfaces.insert(name.to_string(), value);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct InterfaceRuntime{
    pub mtu: Option<u32>,
    pub mac: Option<String>,
    pub ip: Option<String>,
    pub network: Option<String>
}

impl InterfaceRuntime{
    pub fn new(interface_config: &InterfaceConfig, network: Arc<Mutex<NetworkRuntime>>) -> Arc<Mutex<InterfaceRuntime>>{
        let mut network = network.lock().unwrap();
        let ip_address = match &network.network_type{
            NetworkTypeRuntime::Managed{..} => {
                None
            },
            NetworkTypeRuntime::Unmanaged{..} => {
                let (prefix, len_) = network.assign_address().unwrap();
                Some(prefix.to_string())
            },
        };
        
        Arc::new(Mutex::new(InterfaceRuntime{
            mtu: interface_config.mtu,
            mac: Some(generate_mac_address()),
            ip: ip_address,
            network: Some(interface_config.network.clone()),
        }))
    }
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