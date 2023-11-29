use std::collections::HashMap;
use serde_json::json;
use virt::error::Error;
use virt::connect::Connect;
use crate::config::config::UserConfig;
use crate::instance;
use crate::instance::instance::InstanceRuntime;
use handlebars::Handlebars;
use std::process::Command;

pub struct VirtManager{
    pub conn: Connect,
}

impl VirtManager{
    pub fn new() -> VirtManager{
        let uri = "qemu:///system";
        println!("Attempting to connect to hypervisor: '{:?}'", uri);
        let conn = match Connect::open(uri) {
            Ok(c) => c,
            Err(e) => panic!("No connection to hypervisor: {} ", e),
        };
        VirtManager{
            conn,
        }
    }
    pub fn connect(&mut self) {
        match self.conn.get_uri() {
            Ok(u) => println!("Connected to hypervisor at '{}'", u),
            Err(e) => {
                self.disconnect();
                panic!("Failed to get URI for hypervisor connection: {}", e);
            }
        };
    }
    pub fn disconnect(&mut self) {
        if let Err(e) = self.conn.close() {
            panic!("Failed to disconnect from hypervisor: {}", e);
        }
        println!("Disconnected from hypervisor");
    }

    pub fn show_hypervisor_info(&self) -> Result<(), Error> {
        if let Ok(hv_type) = self.conn.get_type() {
            if let Ok(mut hv_ver) = self.conn.get_hyp_version() {
                let major = hv_ver / 1000000;
                hv_ver %= 1000000;
                let minor = hv_ver / 1000;
                let release = hv_ver % 1000;
                println!(
                    "Hypervisor: '{}' version: {}.{}.{}",
                    hv_type, major, minor, release
                );
                return Ok(());
            }
        }
        Err(Error::last_error())
    }

    pub fn create_instance(&self, instances: HashMap<String,InstanceRuntime>, user_config: UserConfig) -> anyhow::Result<()> {
        let base_directory = user_config.base_directory.clone();
      
        let reg = Handlebars::new();
        for (name, instance) in &instances{
            let instance_directory = format!("{}/{}", base_directory, name);
            // if instance_directory exists, remove it
            std::fs::remove_dir_all(instance_directory.clone()).ok();
            std::fs::create_dir_all(instance_directory.clone())?;
            std::fs::copy(instance.image.clone(), format!("{}/{}.img", instance_directory, name))?;
            //std::fs::remove_file(format!("{}/cidata.iso", instance_directory)).ok();
            let key = std::fs::read_to_string(user_config.key_path.clone())?;
            let key = key.trim();
            let user_data = format!("{}",reg.render_template(USER_DATA, &json!(
              {
                "user_name": user_config.user_name,
                "key": key,
                "instance_name": name
              }
            ))?);
            let network_data = format!("{}",reg.render_template(NETWORK_DATA, &json!(
              {
                "interfaces": instance.interfaces,
                "route_tables": instance.route_tables,
              }
            ))?);
            println!("{}", network_data);


            std::fs::write(format!("{}/user-data", instance_directory), user_data.clone())?;
            std::fs::write(format!("{}/meta-data", instance_directory), "")?;
            std::fs::write(format!("{}/network-config", instance_directory), network_data.clone())?;
            println!("{}", user_data);
            let mut cmd = Command::new("/usr/bin/genisoimage");
            cmd.arg("-output")
                .arg(format!("{}/cidata.iso", instance_directory))
                .arg("-volid")
                .arg("cidata")
                .arg("-input-charset")
                .arg("utf-8")
                .arg("-joliet")
                .arg("-rock")
                .arg(format!("{}/meta-data", instance_directory))
                .arg(format!("{}/user-data", instance_directory))
                .arg(format!("{}/network-config", instance_directory));
            cmd.output()?;
        }
        
        let xml = format!("{}",reg.render_template(DOMAIN_DEV, &json!(
          {"config": 
            { 
              "instances": instances, 
              "base_directory": base_directory
            }
          }
        ))?);
        println!("{}", xml);
        virt::domain::Domain::create_xml(&self.conn, &xml, 0)?;
        
        Ok(())
    }
}

const NETWORK_DATA: &str = r#"version: 1
config:
  {{#each interfaces as |interface|}}
  {{#if interface.managed}}
  - type: physical
    name: eth0
    mac_address: '{{interface.mac_address}}'
    subnets:
       - type: dhcp
  {{else}}
  - type: physical
    name: {{@key}}
    mtu: {{interface.mtu}}
    mac_address: '{{interface.mac_address}}'
    subnets:
       - type: static
         address: {{interface.address}}/{{interface.prefix_len}}
  {{/if}}
  {{/each}}
  {{#each route_tables as |route_table|}}
  {{#each routes as |route|}}
  {{#each route as |next_hop|}}
  - type: route
    destination: {{@../key}}
    gateway: {{next_hop}}
    metric: 1
  {{/each}}
  {{/each}}
  {{/each}}
"#;

const USER_DATA: &str = r#"#cloud-config
hostname: {{ instance_name }}
fqdn: {{ instance_name }}
package_update: false
package_upgrade: false
ssh_pwauth: true
disable_root: false
users:
- default
- name: ubuntu
  shell: /bin/bash
  sudo: ALL=(ALL) NOPASSWD:ALL
  lock_passwd: false
  ssh-authorized-keys:
  - {{ key }}
- name: {{ user_name }}
  shell: /bin/bash
  sudo: ALL=(ALL) NOPASSWD:ALL
  lock_passwd: false
  ssh-authorized-keys:
  - {{ key }}
"#;

/*
const USER_DATA: &str = r#"#cloud-config
hostname: {{ instance_name }}
fqdn: {{ instance_name }}
package_update: false
package_upgrade: false
ssh_pwauth: true
disable_root: false
autoinstall:
  updates: security
  apt:
    preferences:
      - package: "*"
        pin: "release a=mantic-security"
        pin-priority: 200
  late-commands:
    - |
      rm /target/etc/apt/preferences.d/90curtin.pref
      true
bootcmd:
- [ snap, remove, --purge, lxd ]
- [ snap, remove, --purge, core20 ]
- [ snap, remove, --purge, snapd ]
- [ apt, --purge, autoremove, snapd ]
ssh-authorized-keys:
- {{ key }}
"#;
*/


const DOMAIN_DEV: &str = r#"

{{#each config.instances as |instance|}}
<domain type="qemu">
  <name>{{ @key }}</name>
  <uuid>8634a4a3-a491-43d4-85c6-5e47489ee0ea</uuid>
  <metadata>
    <libosinfo:libosinfo xmlns:libosinfo="http://libosinfo.org/xmlns/libvirt/domain/1.0">
      <libosinfo:os id="http://ubuntu.com/ubuntu/23.10"/>
    </libosinfo:libosinfo>
  </metadata>
  <memory unit='GiB'>{{ instance.memory }}</memory>
  <currentMemory unit='GiB'>{{ instance.memory }}</currentMemory>
  <vcpu>{{ instance.vcpu }}</vcpu>
  <os>
    <type arch="x86_64" machine="q35">hvm</type>
    <boot dev="hd"/>
  </os>
  <features>
    <acpi/>
    <apic/>
  </features>
  <clock offset="utc">
    <timer name="rtc" tickpolicy="catchup"/>
    <timer name="pit" tickpolicy="delay"/>
    <timer name="hpet" present="no"/>
  </clock>
  <pm>
    <suspend-to-mem enabled="no"/>
    <suspend-to-disk enabled="no"/>
  </pm>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    <disk type="file" device="disk">
      <driver name="qemu" type="qcow2"/>
      <source file="{{../config.base_directory}}/{{@key}}/{{ @key }}.img"/>
      <target dev="vda" bus="virtio"/>
    </disk>
    <controller type="usb" model="qemu-xhci" ports="15"/>
    <controller type="pci" model="pcie-root"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    <controller type="pci" model="pcie-root-port"/>
    {{#each interfaces as |interface|}}
    {{#if interface.managed}}
    <interface type='network'>
        <source network='{{interface.managed}}'/>
        <mac address='{{interface.mac_address}}'/>
        <model type='virtio'/>
    {{else}}
    <interface type='ethernet'>
        <target dev='{{@key}}'/>
        <mac address='{{interface.mac_address}}'/>
        <model type='virtio'/>
    {{/if}}
    </interface>
    {{/each}}
    <console type="pty"/>
    <channel type="unix">
      <source mode="bind"/>
      <target type="virtio" name="org.qemu.guest_agent.0"/>
    </channel>
    <input type="tablet" bus="usb"/>
    <graphics type="vnc" port="-1" listen="0.0.0.0"/>
    <video>
      <model type="vga"/>
    </video>
    <memballoon model="virtio"/>
    <rng model="virtio">
      <backend model="random">/dev/urandom</backend>
    </rng>
    <disk type="file" device="cdrom">
      <driver name="qemu" type="raw"/>
      <source file="{{../config.base_directory}}/{{@key}}/cidata.iso"/>
      <target dev="sda" bus="sata"/>
      <readonly/>
    </disk>
    <serial type='file'>
        <source path='{{../config.base_directory}}/{{@key}}/{{ @key }}.log'/>
    <target port='0'/>
  </serial>
  </devices>
</domain>
{{/each}}"#;