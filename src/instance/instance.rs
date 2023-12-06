use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use serde::{Deserialize, Serialize};
use crate::interface::interface::{InterfaceConfig, InterfaceRuntime};
use crate::object::object::Object;
use crate::config::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceConfig{
    pub vcpu: u16,
    pub memory: String,
    pub image: String,
    pub interfaces: HashMap<String, InterfaceConfig>,
    pub routes: Option<Vec<Route>>,
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Route{
    pub destination: Destination,
    pub next_hops: Vec<NextHop>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Destination{
    pub instance: String,
    pub interface: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NextHop{
    pub instance: String,
    pub interface: String,
}

impl InstanceConfig{
    pub fn new(vcpu: u16, memory: &str, image: &str) -> InstanceConfig{
        InstanceConfig{
            vcpu,
            memory: memory.to_string(),
            image: image.to_string(),
            interfaces: HashMap::new(),
            routes: None,
        }
    }
}

impl <'a>Object<'a, InstanceConfig> for Config {
    fn get(&'a self, name: &str) -> Option<&'a InstanceConfig> {
        self.instances.get(name)
    }
    fn get_mut(&'a mut self, name: &str) -> Option<&'a mut InstanceConfig> {
        self.instances.get_mut(name)
    }
    fn add(&mut self, name: &str, value: InstanceConfig) {
        self.instances.insert(name.to_string(), value);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceRuntime{
    pub vcpu: u16,
    pub memory: String,
    pub image: String,
    pub interfaces: HashMap<String, Arc<Mutex<InterfaceRuntime>>>,
    pub routes: Option<Vec<RouteRuntime>>,
}

impl InstanceRuntime{
    pub fn new(instance_config: &InstanceConfig) -> Arc<Mutex<InstanceRuntime>>{
        Arc::new(Mutex::new(InstanceRuntime{
            vcpu: instance_config.vcpu,
            memory: instance_config.memory.clone(),
            image: instance_config.image.clone(),
            interfaces: HashMap::new(),
            routes: None,
        }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouteRuntime{
    pub destination: String,
    pub next_hops: Vec<NextHopRuntime>,

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NextHopRuntime{
    pub mac_address: String,
    pub ip_address: String,
}
