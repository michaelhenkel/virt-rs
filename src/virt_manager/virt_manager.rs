use std::collections::HashMap;
use serde_json::json;
use virt::error::Error;
use virt::connect::Connect;
use crate::config::config::UserConfig;
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
            let xml = format!("{}",reg.render_template(DOMAIN_DEV, &json!(
              { 
                "name": name,
                "instance": instance, 
                "base_directory": base_directory
              }
            ))?);
            println!("{}", xml);
            virt::domain::Domain::create_xml(&self.conn, &xml, 0)?;
        }
        

        
        Ok(())
    }
}

const NETWORK_DATA: &str = r#"version: 2
ethernets:
  {{#each interfaces as |interface|}}
  {{#if interface.managed}}
  {{@key}}:
    match:
      macaddress: '{{interface.mac_address}}'
    dhcp4: true
  {{else}}
  {{@key}}:
    mtu: {{interface.mtu}}
    match:
      macaddress: '{{interface.mac_address}}'
    addresses:
    - {{interface.address}}/{{interface.prefix_len}}
    {{#if interface.routes}}
    routes:
    {{#each interface.routes as |route|}}
    - to: {{@key}}
      via: {{route}}
      metric: 1
    {{/each}}
    {{/if}}
  {{/if}}
  {{/each}}
"#;

const USER_DATA: &str = r#"#cloud-config
hostname: {{ instance_name }}
fqdn: {{ instance_name }}
package_update: false
package_upgrade: false
ssh_pwauth: true
disable_root: false
bootcmd:
- systemd-machine-id-setup
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

const DOMAIN_DEV: &str = r#"
<domain type="qemu">
  <name>{{ name }}</name>
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
      <source file="{{../base_directory}}/{{name}}/{{ name }}.img"/>
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
    {{#each instance.interfaces as |interface|}}
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
      <source file="{{../base_directory}}/{{name}}/cidata.iso"/>
      <target dev="sda" bus="sata"/>
      <readonly/>
    </disk>
    <serial type='file'>
        <source path='{{../base_directory}}/{{name}}/{{ name }}.log'/>
    <target port='0'/>
  </serial>
  </devices>
</domain>"#;