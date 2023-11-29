pub mod config;
pub mod network;
pub mod instance;
pub mod interface;
pub mod route_table;
pub mod object;
pub mod runtime;
pub mod virt_manager;

use std::collections::HashMap;

use config::config::Config;
use network::network::{NetworkConfig, NetworkTypeConfig};
use instance::instance::InstanceConfig;
use interface::interface::InterfaceConfig;
use object::object::Object;
use runtime::runtime::Runtime;
use virt_manager::virt_manager::VirtManager;
use serde_yaml;
use clap::Parser;

use crate::{route_table::route_table::{RouteTableConfig, InstanceInterface}, config::config::UserConfig};

#[derive(Parser)]
#[clap(version = "0.1.0")]
struct Opts {
    #[clap(long, short)]
    config: Option<String>,
}

fn main() -> anyhow::Result<()>{

    let opts = Opts::parse();
    if let Some(config_file) = opts.config{
        let config = std::fs::read_to_string(config_file).unwrap();
        let config: Config = serde_yaml::from_str(&config).unwrap();
        let serialized = serde_yaml::to_string(&config).unwrap();
        println!("{}", serialized);
        let runtime = Runtime::build(&config);
        let serialized = serde_yaml::to_string(&runtime).unwrap();
        println!("{}", serialized);

        let mut virt_manager = VirtManager::new();
        virt_manager.connect();
        let inst = HashMap::from([(String::from("host1"), runtime.instances.get("host1").unwrap().clone())]);
        virt_manager.create_instance(inst, config.user_config.clone())?;
    } else {
        let mut config = Config::new(Some(UserConfig{
            user_name: "ubuntu".to_string(),
            key_path: "/home/alex/.ssh/id_rsa".to_string(),
        }));

        let network_config = NetworkConfig::new(NetworkTypeConfig::Managed { name: "default".to_string() });
        config.add("mgmt", network_config);

        let network_config = NetworkConfig::new(NetworkTypeConfig::Unmanaged { subnet: "10.0.0.0/24".to_string() });
        config.add("net1", network_config);

        let network_config = NetworkConfig::new(NetworkTypeConfig::Unmanaged { subnet: "10.0.1.0/24".to_string() });
        config.add("net2", network_config);

        let instance_config = InstanceConfig::new(1, 1024, "ubuntu");
        config.add("vm1", instance_config);

        let instance_config = InstanceConfig::new(1, 1024, "ubuntu");
        config.add("vm2", instance_config);

        let interface_config = InterfaceConfig::new(1500, "mgmt", "vm1");
        config.add("vm1_eth0", interface_config);

        let interface_config = InterfaceConfig::new(1500, "net1", "vm1");
        config.add("vm1_eth1", interface_config);

        let interface_config = InterfaceConfig::new(1500, "net2", "vm2");
        config.add("vm2_eth2", interface_config);


        let interface_config = InterfaceConfig::new(1500, "mgmt", "vm2");
        config.add("vm2_eth0", interface_config);

        let interface_config = InterfaceConfig::new(1500, "net1", "vm2");
        config.add("vm2_eth1", interface_config);

        let interface_config = InterfaceConfig::new(1500, "net2", "vm2");
        config.add("vm2_eth2", interface_config);


        let route_table_config = RouteTableConfig::new("vm1", {
            let mut routes = HashMap::new();
            routes.insert("net1".to_string(), vec![InstanceInterface{
                instance: "vm2".to_string(),
                interface: "vm2_eth1".to_string(),
            }]);
            routes.insert("net2".to_string(), vec![InstanceInterface{
                instance: "vm2".to_string(),
                interface: "vm2_eth2".to_string(),
            }]);
            routes
        });

        config.add("vm1_rt1", route_table_config);

        let serialized = serde_yaml::to_string(&config).unwrap();
        println!("{}", serialized);

        let runtime = Runtime::build(&config);

        let serialized = serde_yaml::to_string(&runtime).unwrap();
        println!("{}", serialized);
    }

    Ok(())
}
