pub mod config;
pub mod network;
pub mod instance;
pub mod interface;
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
use clap::{Args, Parser, Subcommand};

use crate::{config::config::UserConfig, interface::interface::InstanceInterface};

#[derive(Parser)]
#[clap(version = "0.1.0")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Create(CommandArgs),
    Destroy(CommandArgs),
    Simulate
}

#[derive(Args)]
struct CommandArgs {
    #[clap(long, short)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Create(opts) => {
            let (config, runtime) = read_config_file(opts.config.as_ref().unwrap())?;
            let mut virt_manager = VirtManager::new();
            virt_manager.connect();
            virt_manager.create_instance(runtime.instances, config.user_config.clone()).await?;
        },
        Commands::Destroy(opts) => {
            let (config, runtime) = read_config_file(opts.config.as_ref().unwrap())?;
            let mut virt_manager = VirtManager::new();
            virt_manager.connect();
            virt_manager.destroy_instance(runtime.instances, config.user_config.clone()).await?;
        },
        Commands::Simulate => {
            let mut config = Config::new(UserConfig{
                user_name: "ubuntu".to_string(),
                key_path: "/home/alex/.ssh/id_rsa".to_string(),
                base_directory: "/var/lib/libvirt/images".to_string(),
            });
    
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
    
            let interface_config = InterfaceConfig::new(1500, "mgmt", "vm1", HashMap::new());
            config.add("vm1_eth0", interface_config);
    
            let interface_config = InterfaceConfig::new(1500, "net1", "vm1", HashMap::from([(
                "net1".to_string(),
                InstanceInterface{
                    instance: "vm2".to_string(),
                    interface: "vm2_eth1".to_string(),
                }
            )]));
            config.add("vm1_eth1", interface_config);
    
            let interface_config = InterfaceConfig::new(1500, "net2", "vm1", HashMap::from([(
                "net1".to_string(),
                InstanceInterface{
                    instance: "vm1".to_string(),
                    interface: "vm1_eth0".to_string(),
                }
            )]));
    
            config.add("vm2_eth2", interface_config);
    
    
            let interface_config = InterfaceConfig::new(1500, "mgmt", "vm2", HashMap::from([(
                "net1".to_string(),
                InstanceInterface{
                    instance: "vm1".to_string(),
                    interface: "vm1_eth0".to_string(),
                }
            )]));
            config.add("vm2_eth0", interface_config);
    
            let interface_config = InterfaceConfig::new(1500, "net1", "vm2", HashMap::from([(
                "net1".to_string(),
                InstanceInterface{
                    instance: "vm1".to_string(),
                    interface: "vm1_eth0".to_string(),
                }
            )]));
    
            config.add("vm2_eth1", interface_config);
    
            let interface_config = InterfaceConfig::new(1500, "net2", "vm2", HashMap::from([(
                "net1".to_string(),
                InstanceInterface{
                    instance: "vm1".to_string(),
                    interface: "vm1_eth0".to_string(),
                }
            )]));
    
            config.add("vm2_eth2", interface_config);
    
            let serialized = serde_yaml::to_string(&config).unwrap();
            println!("{}", serialized);
    
            let runtime = Runtime::build(&config);
    
            let serialized = serde_yaml::to_string(&runtime).unwrap();
            println!("{}", serialized);

        }
    }
    Ok(())
}

fn read_config_file(config_file: &str) -> anyhow::Result<(Config, Runtime)>{
    let config = std::fs::read_to_string(config_file).unwrap();
    let config: Config = serde_yaml::from_str(&config).unwrap();
    //let serialized = serde_yaml::to_string(&config).unwrap();
    //info!("{}", serialized);
    let runtime = Runtime::build(&config);
    //let serialized = serde_yaml::to_string(&runtime).unwrap();
    //info!("{}", serialized);
    Ok((config, runtime))
}
