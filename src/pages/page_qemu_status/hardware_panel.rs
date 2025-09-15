use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use proxmox_yew_comp::http_put;
use serde_json::Value;

use pwt::widget::form::{delete_empty_values, Checkbox, FormContext, Hidden, Number};
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use proxmox_schema::ApiType;

use pwt::prelude::*;
use pwt::widget::{Column, Fa, List, ListTile};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::{PveQmIde, PveQmIdeMedia, QemuConfig, QemuConfigMemory};

use crate::form::{flatten_property_string, property_string_from_parts, pspn};
use crate::widgets::{icon_list_tile, label_field, EditDialog};

#[derive(Clone, PartialEq, Properties)]
pub struct QemuHardwarePanel {
    vmid: u32,
    node: AttrValue,
}

impl QemuHardwarePanel {
    pub fn new(node: impl Into<AttrValue>, vmid: u32) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

fn get_config_url(node: &str, vmid: u32) -> String {
    format!(
        "/nodes/{}/qemu/{}/config",
        percent_encode_component(node),
        vmid
    )
}

fn processor_text(config: &QemuConfig) -> String {
    let cpu = config.cpu.as_deref().unwrap_or("kvm");
    let cores = config.cores.unwrap_or(1);
    let sockets = config.sockets.unwrap_or(1);
    let count = sockets * cores;
    format!(
        "{count} ({}, {}) [{cpu}]",
        tr!("1 Core" | "{n} Cores" % cores),
        tr!("1 Socket" | "{n} Sockets" % sockets)
    )
}

pub enum Msg {
    Load,
    LoadResult(Result<QemuConfig, Error>),
    Dialog(Option<Html>),
    EditMemory,
}

pub struct PveQemuHardwarePanel {
    data: Option<Result<QemuConfig, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    dialog: Option<Html>,
}

// fixme: Number field changes type to Value::String on input!!
fn value_to_u64(value: Value) -> Option<u64> {
    match value {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn read_u64(form_ctx: &FormContext, name: &str) -> Option<u64> {
    let value = form_ctx.read().get_field_value(name.to_string());
    match value {
        Some(value) => value_to_u64(value),
        _ => None,
    }
}

impl PveQemuHardwarePanel {
    fn edit_memory_dialog(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let url = get_config_url(&props.node, props.vmid);

        EditDialog::new(tr!("Memory"))
            .loader(url.clone())
            .on_submit({
                let url = url.clone();
                move |data: Value| http_put(url.clone(), Some(data.clone()))
            })
            .submit_hook(|form_ctx: FormContext| {
                let mut data = form_ctx.get_submit_data();

                if !form_ctx.read().get_field_checked("_use_ballooning") {
                    data["balloon"] = Value::Null;
                    data["shares"] = Value::Null;
                }

                property_string_from_parts::<QemuConfigMemory>(&mut data, "memory", true)?;
                data = delete_empty_values(&data, &["memory", "balloon", "shares"], false);
                Ok(data)
            })
            .load_hook(|mut record| {
                flatten_property_string(&mut record, "memory", &QemuConfigMemory::API_SCHEMA)?;
                let current_memory_prop = pspn("memory", "current");

                let use_ballooning = record["balloon"].as_u64().is_some();
                record["_use_ballooning"] = use_ballooning.into();

                if record["balloon"].is_null() {
                    if let Some(current_memory) = record[current_memory_prop].as_u64() {
                        record["balloon"] = current_memory.into();
                        record["_old_memory"] = current_memory.into();
                    }
                }
                Ok(record)
            })
            .on_done(ctx.link().callback(|_| Msg::Dialog(None)))
            .on_change(|form_ctx: FormContext| {
                let current_memory_prop = pspn("memory", "current");
                let current_memory = form_ctx.read().get_field_value(current_memory_prop.clone());
                let old_memory = form_ctx.read().get_field_value("_old_memory");
                let balloon = form_ctx.read().get_field_value("balloon");

                match (&old_memory, &current_memory, &balloon) {
                    (Some(old_memory), Some(current_memory), Some(balloon)) => {
                        if balloon == old_memory {
                            form_ctx
                                .write()
                                .set_field_value("balloon", current_memory.clone().into());
                        }
                    }
                    _ => {}
                }

                if let Some(current_memory) = current_memory {
                    form_ctx
                        .write()
                        .set_field_value("_old_memory", current_memory.into());
                }
            })
            .renderer(|form_ctx: FormContext, _| {
                let current_memory_prop = pspn("memory", "current");
                let current_memory = read_u64(&form_ctx, &current_memory_prop);

                let use_ballooning = form_ctx.read().get_field_checked("_use_ballooning");

                let disable_shares = {
                    let balloon = read_u64(&form_ctx, "balloon");
                    match (current_memory, balloon) {
                        (Some(memory), Some(balloon)) => memory == balloon,
                        _ => false,
                    }
                };

                let memory_default = 512u64;

                Column::new()
                    .class(pwt::css::FlexFit)
                    .gap(2)
                    .with_child(label_field(
                        tr!("Memory") + " (MiB)",
                        Number::<u64>::new()
                            .name(current_memory_prop)
                            .default(memory_default)
                            .step(32),
                    ))
                    .with_child(Hidden::new().name("_old_memory").submit(false))
                    .with_child(label_field(
                        tr!("Minimum memory") + " (MiB)",
                        Number::<u64>::new()
                            .name("balloon")
                            .submit_empty(true)
                            .disabled(!use_ballooning)
                            .min(1)
                            .max(current_memory)
                            .step(32)
                            .placeholder(current_memory.map(|n| n.to_string())),
                    ))
                    .with_child(label_field(
                        tr!("Shares"),
                        Number::<u64>::new()
                            .name("shares")
                            .submit_empty(true)
                            .disabled(!use_ballooning || disable_shares)
                            .placeholder(tr!("Default") + " (1000)")
                            .max(50000)
                            .step(10),
                    ))
                    .with_child(label_field(
                        tr!("Ballooning Device"),
                        Checkbox::new().name("_use_ballooning").submit(false),
                    ))
                    .into()
            })
            .into()
    }

    fn view_list(&self, ctx: &Context<Self>, data: &QemuConfig) -> Html {
        let mut list: Vec<ListTile> = Vec::new();
        list.push(
            icon_list_tile(
                Fa::new("memory"),
                data.memory.as_deref().unwrap_or("512").to_string() + " MB",
                tr!("Memory"),
                (),
            )
            .interactive(true)
            .on_activate(ctx.link().callback(|_| Msg::EditMemory)),
        );
        list.push(icon_list_tile(
            Fa::new("cpu"),
            processor_text(data),
            tr!("Processor"),
            (),
        ));

        list.push(icon_list_tile(
            Fa::new("microchip"),
            data.bios
                .as_ref()
                .map(|b| b.to_string())
                .unwrap_or(format!("{} (SeaBIOS)", tr!("Default"))),
            tr!("Bios"),
            (),
        ));

        list.push(icon_list_tile(
            Fa::new("gears"),
            data.machine
                .as_ref()
                .map(|b| b.to_string())
                .unwrap_or(format!("{} (i440fx)", tr!("Default")))
                .to_string(),
            tr!("Machine Type"),
            (),
        ));

        for (n, disk_config) in &data.ide {
            if let Ok(config) = crate::form::parse_property_string::<PveQmIde>(disk_config.as_str())
            {
                if config.media == Some(PveQmIdeMedia::Cdrom) {
                    list.push(icon_list_tile(
                        Fa::new("cdrom"),
                        disk_config.to_string(),
                        tr!("CD/DVD Drive") + &format!(" (ide{n})"),
                        (),
                    ));
                } else {
                    list.push(icon_list_tile(
                        Fa::new("hdd-o"),
                        disk_config.to_string(),
                        tr!("Hard Disk") + &format!(" (ide{n})"),
                        (),
                    ));
                }
            }
        }

        for (n, disk_config) in &data.scsi {
            list.push(icon_list_tile(
                Fa::new("hdd-o"),
                disk_config.to_string(),
                tr!("Hard Disk") + &format!(" (scsi{n})"),
                (),
            ));
        }

        for (n, net_config) in &data.net {
            list.push(icon_list_tile(
                Fa::new("exchange"),
                net_config.to_string(),
                tr!("Network Device") + &format!(" (net{n})"),
                (),
            ));
        }

        List::new(list.len() as u64, move |pos| list[pos as usize].clone())
            .grid_template_columns("auto 1fr auto")
            .into()
    }
}

impl Component for PveQemuHardwarePanel {
    type Message = Msg;
    type Properties = QemuHardwarePanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
            reload_timeout: None,
            load_guard: None,
            dialog: None,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.data = None;
        ctx.link().send_message(Msg::Load);
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Dialog(dialog) => {
                if dialog.is_none() && self.dialog.is_some() {
                    ctx.link().send_message(Msg::Load);
                }
                self.dialog = dialog;
            }
            Msg::EditMemory => {
                self.dialog = Some(self.edit_memory_dialog(ctx));
            }
            Msg::Load => {
                let link = ctx.link().clone();
                let url = get_config_url(&props.node, props.vmid);
                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&url, None).await;
                    link.send_message(Msg::LoadResult(result));
                }));
            }
            Msg::LoadResult(result) => {
                self.data = Some(result.map_err(|err| err.to_string()));
                let link = ctx.link().clone();
                self.reload_timeout = Some(Timeout::new(3000, move || {
                    link.send_message(Msg::Load);
                }));
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content =
            crate::widgets::render_loaded_data(&self.data, |data| self.view_list(ctx, data));
        crate::widgets::standard_card(tr!("Hardware"), None::<&str>)
            .min_height(200)
            .with_child(content)
            .with_optional_child(self.dialog.clone())
            .into()
    }
}

impl From<QemuHardwarePanel> for VNode {
    fn from(props: QemuHardwarePanel) -> Self {
        let comp = VComp::new::<PveQemuHardwarePanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
