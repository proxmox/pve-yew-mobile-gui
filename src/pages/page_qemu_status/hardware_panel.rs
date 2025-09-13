use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use proxmox_schema::ApiType;
use pwt::prelude::*;
use pwt::widget::{Fa, List, ListTile};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::{PveQmIde, PveQmIdeMedia, QemuConfig};

use crate::widgets::icon_list_tile;

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
}

pub struct PveQemuHardwarePanel {
    data: Option<Result<QemuConfig, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
}

impl PveQemuHardwarePanel {
    fn view_list(&self, _ctx: &Context<Self>, data: &QemuConfig) -> Html {
        let mut list: Vec<ListTile> = Vec::new();
        list.push(icon_list_tile(
            Fa::new("memory"),
            data.memory.as_deref().unwrap_or("512").to_string() + " MB",
            tr!("Memory"),
            (),
        ));
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
            .into()
    }
}

impl From<QemuHardwarePanel> for VNode {
    fn from(props: QemuHardwarePanel) -> Self {
        let comp = VComp::new::<PveQemuHardwarePanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
