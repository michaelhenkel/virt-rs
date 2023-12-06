use std::{process::{Command, Output}, sync::{Mutex, Arc}, thread::sleep};
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use virt::{network, interface};

use crate::{runtime::runtime::Runtime, instance::{instance::InstanceRuntime, self}, network::network::{NetworkRuntime, NetworkTypeRuntime}, interface::interface::InterfaceRuntime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LxdManager{
    runtime: Runtime,
}

impl LxdManager{
    pub fn new(runtime: Runtime) -> LxdManager{
        LxdManager{
            runtime
        }
    }
    pub fn run(&mut self) -> anyhow::Result<()>{
        /*
        for (name, network) in &self.runtime.networks {
            {
                let network_clone = network.clone();
                let network_clone = network_clone.lock().unwrap();
                match network_clone.network_type{
                    NetworkTypeRuntime::Managed{..} => {
                        continue;
                    },
                    NetworkTypeRuntime::Unmanaged{..} => {

                    },
                }
            }

            let command = LxdCommands::CreateNetwork{
                name: name.clone(),
                config: network.clone(),
            };
            if let Err(e) = command.command(){
                return Err(anyhow::Error::new(e));
            }
        }
        */
        for (name, instance) in &self.runtime.instances {
            let command = LxdCommands::LaunchInstance{
                name: name.clone(),
                config: instance.clone(),
            };
            if let Err(e) = command.command(){
                return Err(anyhow::Error::new(e));
            }
            let instance = instance.clone();
            let instance = instance.lock().unwrap();
            let mut idx: usize = 1;
            for (intf_name, interface) in &instance.interfaces {
                {
                    let interface = interface.clone();
                    let interface = interface.lock().unwrap();
                    if interface.ip.is_none(){
                        continue;
                    }
                }
                let command = LxdCommands::AttachInterface{
                    name: intf_name.clone(),
                    config: interface.clone(),
                    instance: name.clone(),
                    idx,
                };
                
                if let Err(e) = command.command(){
                    return Err(anyhow::Error::new(e));
                }
                idx += 1;
                sleep(std::time::Duration::from_secs(2));
            }
            sleep(std::time::Duration::from_secs(2));
        }
        Ok(())
    }
    pub fn destroy(&mut self) -> anyhow::Result<()>{
        for (name, _instance) in &self.runtime.instances {
            let command = LxdCommands::DestroyInstance{
                name: name.clone(),
            };
            if let Err(e) = command.command(){
                log::info!("Error destroying instance {}: {}", name, e);
                continue;
            }
        }
        /*
        for (name, network) in &self.runtime.networks {
            {
                let network = network.lock().unwrap();
                match network.network_type{
                    NetworkTypeRuntime::Managed{..} => {
                        continue;
                    },
                    NetworkTypeRuntime::Unmanaged{..} => {

                    },
                }
            }
            let command = LxdCommands::DestroyNetwork{
                name: name.clone(),
            };
            if let Err(e) = command.command(){
                log::info!("Error destroying network {}: {}", name, e);
                continue;
            }

        }
        */
        Ok(())
    }
}

//lxc launch images:ubuntu/23.04/default build2 --vm -s pool1 -c limits.cpu=4 -c limits.memory=24GB

enum LxdCommands{
    LaunchInstance{
        name: String,
        config: Arc<Mutex<InstanceRuntime>>
    },
    CreateNetwork{
        name: String,
        config: Arc<Mutex<NetworkRuntime>>
    },
    DestroyInstance{
        name: String,
    },
    DestroyNetwork{
        name: String,
    },
    AttachInterface{
        name: String,
        config: Arc<Mutex<InterfaceRuntime>>,
        instance: String,
        idx: usize
    }
}

impl LxdCommands{
    pub fn command(&self) -> Result<Output, std::io::Error>{
        match self {
            LxdCommands::LaunchInstance { name, config } => {
                log::info!("Launching instance {}", name);
                let config = config.lock().unwrap();
                let mut cmd = Command::new("lxc");
                cmd.arg("launch").
                    arg(&config.image).
                    arg(&name).
                    arg("--vm").
                    arg("-c").
                    arg(format!("limits.cpu={}", config.vcpu)).
                    arg("-c").
                    arg(format!("limits.memory={}", config.memory));
                let res = cmd.output();
                match res {
                    Ok(res) => {
                        if !res.status.success(){
                            let stderr = std::str::from_utf8(&res.stderr).unwrap();
                            return Err(io::Error::new(io::ErrorKind::Other, stderr));
                        } else {
                            return Ok(res);
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            LxdCommands::DestroyInstance { name } => {
                log::info!("Destroying instance {}", name);
                let mut cmd = Command::new("lxc");
                cmd.arg("delete");
                cmd.arg(&name);
                cmd.arg("--force");
                let res = cmd.output();
                match res {
                    Ok(res) => {
                        if !res.status.success(){
                            let stderr = std::str::from_utf8(&res.stderr).unwrap();
                            return Err(io::Error::new(io::ErrorKind::Other, stderr));
                        } else {
                            return Ok(res);
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            LxdCommands::CreateNetwork { name, config } => {
                log::info!("Creating network {}", name);
                let config_lock = config.lock().unwrap();
                let (subnet, gateway) = {
                    //let config = config.clone();
                    match &config_lock.network_type{
                        NetworkTypeRuntime::Managed{..} => {
                            return Err(io::Error::new(io::ErrorKind::Other, "Cannot create managed network"));
                        },
                        NetworkTypeRuntime::Unmanaged{subnet, assigned_addresses: _, gateway} => {
                            (subnet, gateway.unwrap())
                        }
                    
                    }
                };
                let mut cmd = Command::new("lxc");
                cmd.arg("network").
                    arg("create").
                    arg(&name).
                    arg("--type").
                    arg("bridge").
                    arg(format!("ipv4.address={}/24",gateway.to_string())).
                    arg(format!("ipv4.dhcp.gateway={}", gateway.to_string()));

                let res = cmd.output();
                match res {
                    Ok(res) => {
                        if !res.status.success(){
                            log::info!("{:?}", cmd);
                            let stderr = std::str::from_utf8(&res.stderr).unwrap();
                            return Err(io::Error::new(io::ErrorKind::Other, stderr));
                        } else {
                            return Ok(res);
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }


            },
            LxdCommands::DestroyNetwork { name } => {
                log::info!("Destroying network {}", name);
                let mut cmd = Command::new("lxc");
                cmd.arg("network").
                    arg("delete").
                    arg(&name);
                let res = cmd.output();
                match res {
                    Ok(res) => {
                        if !res.status.success(){
                            let stderr = std::str::from_utf8(&res.stderr).unwrap();
                            return Err(io::Error::new(io::ErrorKind::Other, stderr));
                        } else {
                            return Ok(res);
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            },
            LxdCommands::AttachInterface { name, config , instance, idx} => {
                log::info!("Attaching interface {} to instance {}", name, instance);
                let config = config.lock().unwrap();
                let mut cmd = Command::new("lxc");
                cmd.arg("config").
                    arg("device").
                    arg("add").
                    arg(&instance).
                    arg(format!("eth{}", idx)).
                    arg("nic").
                    arg(format!("name={}", name)).
                    arg(format!("nictype=routed")).
                    arg(format!("ipv4.address={}", config.ip.as_ref().unwrap())).
                    arg(format!("hwaddr={}", config.mac.as_ref().unwrap())).
                    arg(format!("mtu={}", config.mtu.as_ref().unwrap())).
                    arg("ipv4.gateway=none").
                    arg("ipv6.gateway=none").
                    arg(format!("host_name={}_{}", instance, name));
                let res = cmd.output();
                match res {
                    Ok(res) => {
                        if !res.status.success(){
                            let stderr = std::str::from_utf8(&res.stderr).unwrap();
                            return Err(io::Error::new(io::ErrorKind::Other, stderr));
                        } else {
                            return Ok(res);
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
    }
}
//lxc config device add CONTAINER-NAME eth1 nic name=eth1 nictype=bridged parent=lxdbr0


//lxc network create test --type bridge