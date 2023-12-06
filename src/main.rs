pub mod config;
pub mod network;
pub mod instance;
pub mod interface;
pub mod object;
pub mod runtime;
pub mod lxd_manager;

use config::config::Config;
use runtime::runtime::Runtime;
use network::network::{NetworkConfig, NetworkTypeConfig};
use instance::instance::{InstanceConfig, Route};
use interface::interface::InterfaceConfig;
use object::object::Object;
use lxd_manager::lxd_manager::LxdManager;
use serde_yaml;
use clap::{Args, Parser, Subcommand};

use crate::config::config::UserConfig;
use crate::instance::instance::{Destination, NextHop};

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

//#[tokio::main]
fn main() -> anyhow::Result<()>{
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Create(opts) => {
            let config = read_config_file(opts.config.as_ref().unwrap())?;
            let runtime = Runtime::build(&config);
            let serialized = serde_yaml::to_string(&runtime).unwrap();
            println!("{}", serialized);
            let mut lxd_manager = LxdManager::new(runtime);
            lxd_manager.run()?;
        },
        Commands::Destroy(opts) => {
            let config = read_config_file(opts.config.as_ref().unwrap())?;
            let runtime = Runtime::build(&config);
            let mut lxd_manager = LxdManager::new(runtime);
            lxd_manager.destroy()?;
        },
        Commands::Simulate => {
            let mut config = Config::new(UserConfig{
                user_name: "ubuntu".to_string(),
                key_path: "/home/alex/.ssh/id_rsa".to_string(),
                base_directory: "/var/lib/libvirt/images".to_string(),
            });
    
            let mgmt_network = NetworkConfig::new(NetworkTypeConfig::Managed { name: "default".to_string() });
            
    
            let net1 = NetworkConfig::new(NetworkTypeConfig::Unmanaged { 
                subnet: "10.0.0.0/24".to_string(),
            });
            
    
            let net2 = NetworkConfig::new(NetworkTypeConfig::Unmanaged {
                subnet: "10.0.1.0/24".to_string(),
            });
            
    
            let mut host1 = InstanceConfig::new(1, "2GB", "ubuntu");
            let mut host2 = InstanceConfig::new(1, "2GB", "ubuntu");
            
            let host1_eth0 = InterfaceConfig::new(Some(1500), "net1");
            let host2_eth0 = InterfaceConfig::new(Some(1500), "net2");

            host1.routes = Some(vec![
                Route{
                    destination: Destination{
                        instance: "host2".to_string(),
                        interface: "eth0".to_string(),
                    },
                    next_hops: vec![
                        NextHop{
                            instance: "host2".to_string(),
                            interface: "eth0".to_string(),
                        }
                    ]
                }
            ]);
    

            host1.add("eth0", host1_eth0);
            host2.add("eth0", host2_eth0);


            config.add("mgmt", mgmt_network);
            config.add("net1", net1);
            config.add("net2", net2);
            config.add("host1", host1);
            config.add("host2", host2);

            let serialized = serde_yaml::to_string(&config).unwrap();
            println!("{}", serialized);

            let runtime = Runtime::build(&config);
            let serialized = serde_yaml::to_string(&runtime).unwrap();
            println!("{}", serialized);


        }
    }
    Ok(())
}

fn read_config_file(config_file: &str) -> anyhow::Result<Config>{
    let config = std::fs::read_to_string(config_file).unwrap();
    let config: Config = serde_yaml::from_str(&config).unwrap();
    Ok(config)
}
