use std::{collections::HashMap, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};

use crate::{config::config::{UserConfig, Config}, network::network::NetworkRuntime, instance::instance::{InstanceRuntime, NextHopRuntime, RouteRuntime}, interface::interface::InterfaceRuntime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Runtime{
    pub user_config: UserConfig,
    pub networks: HashMap<String, Arc<Mutex<NetworkRuntime>>>,
    pub instances: HashMap<String, Arc<Mutex<InstanceRuntime>>>,
}

impl Runtime{
    pub fn build(config: &Config) -> Runtime{
        let mut networks = HashMap::new();
        let mut instances = HashMap::new();
        let mut interfaces = HashMap::new();
        for (name, network) in &config.networks {
            networks.insert(name.to_string(), NetworkRuntime::new(network));
        }
        for (instance_name, instance) in &config.instances {
            let runtime_instance = InstanceRuntime::new(instance);
            instances.insert(instance_name.to_string(), runtime_instance.clone());
            for (interface_name, interface) in &instance.interfaces {
                let network = networks.get(&interface.network).unwrap();
                let intf = InterfaceRuntime::new(interface, network.clone());
                interfaces.insert((instance_name.clone(), interface_name.to_string()), intf.clone());
                {
                    let mut runtime_instance = runtime_instance.lock().unwrap();
                    runtime_instance.interfaces.insert(interface_name.to_string(), intf.clone());
                }
            }


        }
        for (instance_name, config_instance) in &config.instances {
            let runtime_instance = instances.get(instance_name).unwrap();
            if let Some(routes) = &config_instance.routes{
                let mut runtime_routes = Vec::new();
                for route in routes {
                    let dst_inst = route.destination.instance.clone();
                    let dst_intf = route.destination.interface.clone();
                    let dst_intf = interfaces.get(&(dst_inst, dst_intf)).unwrap();
                    let mut next_hops = Vec::new();
                    for next_hop in &route.next_hops {
                        let next_hop_inst = next_hop.instance.clone();
                        let next_hop_intf = next_hop.interface.clone();
                        let next_hop_intf = interfaces.get(&(next_hop_inst, next_hop_intf)).unwrap();
                        let mac_address = {
                            let next_hop_intf = next_hop_intf.lock().unwrap();
                            next_hop_intf.mac.clone().unwrap()
                        };
                        let ip_address = {
                            let next_hop_intf = next_hop_intf.lock().unwrap();
                            next_hop_intf.ip.clone().unwrap()
                        };
                        let next_hop_runtime = NextHopRuntime{
                            mac_address,
                            ip_address,
                        };
                        next_hops.push(next_hop_runtime);
                    }
                    let dst_ip = {
                        let dst_intf = dst_intf.lock().unwrap();
                        dst_intf.ip.clone().unwrap()
                    };
                    let route_runtime = RouteRuntime{
                        destination: dst_ip,
                        next_hops,
                    };
                    runtime_routes.push(route_runtime);
                }
                {
                    let mut runtime_instance = runtime_instance.lock().unwrap();
                    runtime_instance.routes = Some(runtime_routes);
                }
            }

        }
        Runtime{
            user_config: config.user_config.clone(),
            networks,
            instances,
        }
    }
}