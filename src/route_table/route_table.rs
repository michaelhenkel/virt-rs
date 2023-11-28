use std::collections::HashMap;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use crate::instance::instance::InstanceRuntime;
use crate::network::network::NetworkRuntime;
use crate::object::object::Object;
use crate::config::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouteTableConfig{
    pub routes: HashMap<String, Vec<InstanceInterface>>,
    pub instance: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceInterface{
    pub instance: String,
    pub interface: String,
}

impl RouteTableConfig{
    pub fn new(instance: &str, routes: HashMap<String, Vec<InstanceInterface>>) -> RouteTableConfig{
        RouteTableConfig{
            instance: instance.to_string(),
            routes,
        }
    }
}

impl <'a>Object<'a, RouteTableConfig> for Config {
    fn get(&'a self, name: &str) -> Option<&'a RouteTableConfig> {
        self.route_tables.get(name)
    }
    fn get_mut(&'a mut self, name: &str) -> Option<&'a mut RouteTableConfig> {
        self.route_tables.get_mut(name)
    }
    fn add(&mut self, name: &str, value: RouteTableConfig) {
        self.route_tables.insert(name.to_string(), value);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouteTableRuntime{
    pub routes: HashMap<ipnet::Ipv4Net, Vec<Ipv4Addr>>,
}

impl RouteTableRuntime{
    pub fn configure(config: &Config, networks: &HashMap<String, NetworkRuntime>, instances: &mut HashMap<String, InstanceRuntime>){
        for (name, route_table) in &config.route_tables{
            let instances_clone = instances.clone();
            if let Some(instance) = instances.get_mut(&route_table.instance){
                let mut routes = HashMap::new();
                for (destination, next_hops) in &route_table.routes{
                    if let Some(network) = networks.get(destination){
                        for next_hop in next_hops{
                            if let Some(next_hop_instance) = instances_clone.get(&next_hop.instance){
                                if let Some(next_hop_interface) = next_hop_instance.interfaces.get(&next_hop.interface){
                                    let destination = network.subnet;
                                    let next_hop = next_hop_interface.address;
                                    routes.entry(destination).or_insert(Vec::new()).push(next_hop);
                                }
                            }
                        }
                    }
                }
                if routes.len() > 0{
                    let route_table = RouteTableRuntime{
                        routes,
                    };
                    instance.route_tables.insert(name.clone(), route_table);
                }
            }
        }
    }
}