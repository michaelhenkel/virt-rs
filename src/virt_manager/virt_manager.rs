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

    pub fn create_instance(&self, instances: HashMap<String,InstanceRuntime>, user_config: Option<UserConfig>) -> anyhow::Result<()> {
        let reg = Handlebars::new();
        let xml = format!("{}",reg.render_template(DOMAIN_DEV, &instances)?);
        println!("{}", xml);
        for (name, instance) in instances{
            std::fs::copy(instance.image, format!("/var/lib/libvirt/images/{}.img", name))?;
        }
        if let Some(user_config) = user_config{
            std::fs::remove_file("/var/lib/libvirt/images/cidata.iso").ok();
            let key = std::fs::read_to_string(user_config.key_path)?;
            let key = key.trim();
            let user_data = format!("{}",reg.render_template(USER_DATA, &json!({"user_name": user_config.user_name, "key": key}))?);
            std::fs::write("/var/lib/libvirt/images/user-data", user_data.clone())?;
            std::fs::write("/var/lib/libvirt/images/meta-data", "")?;
            println!("{}", user_data);
            let mut cmd = Command::new("/usr/bin/genisoimage");
            cmd.arg("-output")
                .arg("/var/lib/libvirt/images/cidata.iso")
                .arg("-volid")
                .arg("cidata")
                .arg("-input-charset")
                .arg("utf-8")
                .arg("-joliet")
                .arg("-rock")
                .arg("/var/lib/libvirt/images/meta-data")
                .arg("/var/lib/libvirt/images/user-data");
            cmd.output()?;
            //genisoimage -output ci.iso -quiet -volid cidata -input-charset utf-8 -joliet -rock user-data meta-data network-config
        }
        virt::domain::Domain::create_xml(&self.conn, &xml, 0)?;
        Ok(())
    }
}

const USER_DATA: &str = r#"#cloud-config
hostname: cl-ubuntu
fqdn: cl-ubuntu
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


const DOMAIN_DEV: &str = r#"
{{#each this as |instance|}}
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
      <source file="/var/lib/libvirt/images/{{ @key }}.img"/>
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
        <model type='virtio'/>
    {{else}}
    <interface type='ethernet'>
        <target dev='{{@key}}'/>
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
      <source file="/var/lib/libvirt/images/cidata.iso"/>
      <target dev="sda" bus="sata"/>
      <readonly/>
    </disk>
    <serial type='file'>
        <source path='/var/lib/libvirt/images/{{ @key }}.log'/>
    <target port='0'/>
  </serial>
  </devices>
</domain>
{{/each}}"#;