use std::rc::Rc;

use anyhow::Error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use proxmox_schema::ApiType;
use pve_api_types::PveQmBoot;

use pwt::prelude::*;
use pwt::widget::form::{
    Checkbox, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
};
use pwt::widget::{ActionIcon, List, ListTile, Row};

use pwt_macros::{builder, widget};

pub type PveBootDeviceList = ManagedFieldMaster<PveBootDeviceField>;

#[widget(comp=ManagedFieldMaster<PveBootDeviceField>, @input)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct BootDeviceList {
    qemu_config: Rc<Value>,
}

impl BootDeviceList {
    pub fn new(qemu_config: Rc<Value>) -> Self {
        yew::props!(Self { qemu_config })
    }
}

pub enum Msg {
    Up(usize),
    Down(usize),
    Enable(usize, bool),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeviceEntry {
    enabled: bool,
    name: String,
    value: Option<String>,
}

#[doc(hidden)]
pub struct PveBootDeviceField {
    boot_devices: Vec<(String /* key */, String /* value */)>,
    devices: Vec<DeviceEntry>,
}

// add disabled devices as well
fn add_disabled_devices(list: &mut Vec<DeviceEntry>, boot_devices: &[(String, String)]) {
    let mut disabled_list = Vec::new();
    for (device, value) in boot_devices {
        if list.iter().find(|i| &i.name == device).is_none() {
            disabled_list.push(DeviceEntry {
                enabled: false,
                name: device.clone(),
                value: Some(value.clone()),
            })
        }
    }
    disabled_list.sort_by_key(|i| i.name.clone());
    list.extend(disabled_list);
}

impl PveBootDeviceField {
    pub fn update_device_description(&mut self, qemu_config: &Value) {
        let lookup_value = |name| {
            qemu_config
                .get(name)
                .map(|v| v.as_str().map(String::from))
                .flatten()
        };

        for item in self.devices.iter_mut() {
            item.value = lookup_value(&item.name);
        }
    }

    pub fn update_device_list(&mut self, value: Value, qemu_config: &Value) {
        let value = match value {
            Value::String(s) => s,
            Value::Array(_) => {
                return; // internal state, no update necessary
            }
            _ => {
                log::error!("unable to parse boot property string: got wrong type");
                String::new()
            }
        };

        let boot: PveQmBoot = match crate::form::parse_property_string(&value) {
            Ok(boot) => boot,
            Err(err) => {
                log::error!("unable to parse boot option value: {err}");
                PveQmBoot {
                    legacy: Some(String::from("cdn")),
                    order: None,
                }
            }
        };

        let mut list: Vec<DeviceEntry> = if let Some(order) = &boot.order {
            order
                .clone()
                .split(";")
                .filter(|name| !name.is_empty())
                .map(|name| DeviceEntry {
                    enabled: true,
                    name: name.to_string(),
                    value: None,
                })
                .collect()
        } else if let Some(mut order) = boot.legacy.as_deref() {
            // legacy style, transform to new bootorder
            if order.is_empty() {
                order = "cdn"
            }
            let bootdisk: Option<&str> = qemu_config.get("bootdisk").map(|v| v.as_str()).flatten();

            let mut list = Vec::new();
            for c in order.chars() {
                match c {
                    'c' => {
                        if let Some(bootdisk) = bootdisk {
                            list.push(DeviceEntry {
                                name: bootdisk.to_string(),
                                value: None,
                                enabled: true,
                            });
                        }
                    }
                    'd' => {
                        if let Some(map) = qemu_config.as_object() {
                            for (k, v) in map {
                                if let Some(v) = v.as_str() {
                                    if v.contains("media=cdrom") && is_disk(k) && !is_cloud_init(v)
                                    {
                                        list.push(DeviceEntry {
                                            name: k.clone(),
                                            value: None,
                                            enabled: true,
                                        })
                                    }
                                }
                            }
                        }
                    }
                    'n' => {
                        if let Some(map) = qemu_config.as_object() {
                            for key in map.keys() {
                                if is_net_device(key) {
                                    list.push(DeviceEntry {
                                        name: key.clone(),
                                        value: None,
                                        enabled: true,
                                    })
                                }
                            }
                        }
                    }
                    other => {
                        log::error!("ignore unknown legacy boot order {other:?}");
                    }
                }
            }
            list
        } else {
            Vec::new()
        };

        self.update_device_description(qemu_config);
        add_disabled_devices(&mut list, &self.boot_devices);

        self.devices = list;
    }
}

impl ManagedField for PveBootDeviceField {
    type Message = Msg;
    type Properties = BootDeviceList;
    type ValidateClosure = ();

    fn validation_args(_props: &Self::Properties) -> Self::ValidateClosure {
        ()
    }

    fn validator(_props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        let value = match value {
            Value::Array(_) => {
                let devices: Vec<DeviceEntry> = serde_json::from_value(value.clone()).unwrap();
                let list = devices
                    .into_iter()
                    .filter(|item| item.enabled)
                    .map(|item| item.name)
                    .collect::<Vec<String>>()
                    .join(";");
                Value::from(format!("order={list}"))
            }
            _ => value.clone(),
        };

        Ok(value)
    }
    fn setup(_props: &BootDeviceList) -> ManagedFieldState {
        //let input_props = props.as_input_props();
        let value = Value::Null;
        let default = Value::Null;

        let valid = Ok(());

        ManagedFieldState {
            value,
            valid,
            default,
            radio_group: false,
            unique: false,
        }
    }

    fn create(ctx: &ManagedFieldContext<Self>) -> Self {
        let props = ctx.props();
        //let input_props = props.as_input_props();

        let boot_devices = extract_boot_device_list(&props.qemu_config);
        Self {
            boot_devices,
            devices: Vec::new(),
        }
    }

    fn value_changed(&mut self, ctx: &ManagedFieldContext<Self>) {
        let props = ctx.props();
        let state = ctx.state();
        self.update_device_list(state.value.clone(), &props.qemu_config);
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Enable(pos, enabled) => {
                self.devices[pos].enabled = enabled;
            }
            Msg::Up(pos) => {
                if pos == 0 {
                    return false;
                }
                let new_pos = pos - 1;
                let tmp = self.devices[new_pos].clone();
                self.devices[new_pos] = self.devices[pos].clone();
                self.devices[pos] = tmp;
            }
            Msg::Down(pos) => {
                if (pos + 1) >= self.devices.len() {
                    return false;
                }
                let new_pos = pos + 1;
                let tmp = self.devices[new_pos].clone();
                self.devices[new_pos] = self.devices[pos].clone();
                self.devices[pos] = tmp;
            }
        }

        ctx.link()
            .update_value(serde_json::to_value(&self.devices).unwrap());
        true
    }

    fn changed(&mut self, ctx: &ManagedFieldContext<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.qemu_config != old_props.qemu_config {
            self.boot_devices = extract_boot_device_list(&props.qemu_config);
            self.update_device_description(&props.qemu_config);
            add_disabled_devices(&mut self.devices, &self.boot_devices);
        }
        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let tiles: Vec<ListTile> = self
            .devices
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let enabled = item.enabled;
                let is_last = (index + 1) == self.devices.len();
                let leading: Html = Checkbox::new()
                    .switch(true)
                    .checked(item.enabled)
                    .on_input(ctx.link().callback(move |_| Msg::Enable(index, !enabled)))
                    .into();
                let trailing: Html = Row::new()
                    .with_child(
                        ActionIcon::new("fa fa-chevron-up")
                            .disabled(index == 0)
                            .on_activate(ctx.link().callback(move |_| Msg::Up(index))),
                    )
                    .with_child(
                        ActionIcon::new("fa fa-chevron-down")
                            .disabled(is_last)
                            .on_activate(ctx.link().callback(move |_| Msg::Down(index))),
                    )
                    .into();
                crate::widgets::standard_list_tile(
                    item.name.clone(),
                    item.value.clone(),
                    leading,
                    trailing,
                )
                .interactive(true)
                .border_bottom(!is_last)
                .padding_x(0)
                .key(item.name.clone())
            })
            .collect();
        List::new(tiles.len() as u64, move |pos| tiles[pos as usize].clone())
            .class(pwt::css::FlexFit)
            .grid_template_columns("auto 1fr auto")
            .into()
    }
}

fn is_disk(dev: &str) -> bool {
    thread_local! {
        static BUS_MATCH: Regex = Regex::new(r#"^(ide|sata|virtio|scsi)(\d+)$"#).unwrap();
    }
    BUS_MATCH.with(|r| r.is_match(dev))
}

fn is_net_device(dev: &str) -> bool {
    thread_local! {
        static NET_NAME: Regex = Regex::new(r#"^net\d+$"#).unwrap();
    }
    NET_NAME.with(|r| r.is_match(dev))
}

fn is_cloud_init(v: &str) -> bool {
    thread_local! {
        static CI_VOLUME_NAME: Regex = Regex::new(r#"/[:/]vm-\d+-cloudinit"#).unwrap();
    }
    v.contains("media=cdrom") && CI_VOLUME_NAME.with(|r| r.is_match(v))
}

fn extract_boot_device_list(record: &Value) -> Vec<(String, String)> {
    let mut list = Vec::new();

    thread_local! {
        static HOSTPCI_NAME: Regex = Regex::new(r#"^hostpci\d+$"#).unwrap();
        static USB_NAME: Regex = Regex::new(r#"/^usb\d+$"#).unwrap();
    }

    let is_boot_device = |dev, value: &str| {
        (is_disk(dev) && !is_cloud_init(value))
            || is_net_device(dev)
            || HOSTPCI_NAME.with(|r| r.is_match(dev))
            || (USB_NAME.with(|r| r.is_match(dev)) && !value.contains("spice"))
    };

    if let Some(map) = record.as_object() {
        for (key, v) in map {
            if let Some(value) = v.as_str() {
                if is_boot_device(key, value) {
                    list.push((key.clone(), value.to_string()));
                }
            }
        }
    }

    list
}
