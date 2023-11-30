use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::interface::interface::InterfaceRuntime;
use crate::object::object::Object;
use crate::config::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceConfig{
    pub vcpu: u16,
    pub memory: u16,
    pub image: String,
}

impl InstanceConfig{
    pub fn new(vcpu: u16, memory: u16, image: &str) -> InstanceConfig{
        InstanceConfig{
            vcpu,
            memory,
            image: image.to_string(),
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
    pub memory: u16,
    pub image: String,
    pub interfaces: HashMap<String, InterfaceRuntime>,
}

impl From<InstanceConfig> for InstanceRuntime{
    fn from(config: InstanceConfig) -> Self {
        let vcpu = config.vcpu;
        let memory = config.memory;
        let image = config.image;
        let interfaces = HashMap::new();
        InstanceRuntime{
            vcpu,
            memory,
            image,
            interfaces,
        }
    }
}

impl From<&Config> for HashMap<String,InstanceRuntime>{
    fn from(config: &Config) -> Self {
        let mut instances = HashMap::new();
        for (name, instance) in &config.instances{
            instances.insert(name.to_string(), InstanceRuntime::from(instance.clone()));
        }
        instances
    }
}