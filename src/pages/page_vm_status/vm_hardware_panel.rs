use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use proxmox_schema::ApiType;
use pwt::prelude::*;
use pwt::widget::{Fa, List, ListTile, Progress};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::{PveQmIde, PveQmIdeMedia, QemuConfig};

use crate::widgets::icon_list_tile;

#[derive(Clone, PartialEq, Properties)]
pub struct VmHardwarePanel {
    vmid: u32,
    node: AttrValue,
}

impl VmHardwarePanel {
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
    format!("{count} ({sockets} sockets, {cores} cores) [{cpu}]")
}

pub enum Msg {
    Load,
    LoadResult(Result<QemuConfig, Error>),
}

pub struct PveVmHardwarePanel {
    data: Option<Result<QemuConfig, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
}

impl PveVmHardwarePanel {
    fn view_list(&self, _ctx: &Context<Self>, data: &QemuConfig) -> Html {
        let mut list: Vec<ListTile> = Vec::new();
        list.push(icon_list_tile(
            Fa::new("memory"),
            data.memory.as_deref().unwrap_or("-").to_string(),
            "Memory",
            None,
        ));
        list.push(icon_list_tile(
            Fa::new("cpu"),
            processor_text(data),
            "Processor",
            None,
        ));

        list.push(icon_list_tile(
            Fa::new("microchip"),
            data.bios
                .map(|b| b.to_string())
                .unwrap_or(String::from("Default (SeaBIOS)")),
            "Bios",
            None,
        ));

        list.push(icon_list_tile(
            Fa::new("gears"),
            data.machine
                .as_deref()
                .unwrap_or("Default (i440fx)")
                .to_string(),
            "Machine Type",
            None,
        ));

        for (n, disk_config) in &data.ide {
            if let Ok(config) = PveQmIde::API_SCHEMA.parse_property_string(disk_config) {
                if let Ok(config) = serde_json::from_value::<PveQmIde>(config) {
                    if config.media == Some(PveQmIdeMedia::Cdrom) {
                        list.push(icon_list_tile(
                            Fa::new("cdrom"),
                            disk_config.to_string(),
                            format!("CD/DVD Drive (ide{n})"),
                            None,
                        ));
                    } else {
                        list.push(icon_list_tile(
                            Fa::new("hdd-o"),
                            disk_config.to_string(),
                            format!("Hard Disk (ide{n})"),
                            None,
                        ));
                    }
                }
            }
        }

        for (n, disk_config) in &data.scsi {
            list.push(icon_list_tile(
                Fa::new("hdd-o"),
                disk_config.to_string(),
                format!("Hard Disk (scsi{n})"),
                None,
            ));
        }

        for (n, net_config) in &data.net {
            list.push(icon_list_tile(
                Fa::new("exchange"),
                net_config.to_string(),
                format!("Network Device (new{n})"),
                None,
            ));
        }

        List::new(list.len() as u64, move |pos| list[pos as usize].clone())
            .grid_template_columns("auto 1fr auto")
            .into()
    }
}

impl Component for PveVmHardwarePanel {
    type Message = Msg;
    type Properties = VmHardwarePanel;

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
        let content = match &self.data {
            Some(Ok(data)) => self.view_list(ctx, data),
            Some(Err(err)) => pwt::widget::error_message(err).into(),
            None => Progress::new().class("pwt-delay-visibility").into(),
        };

        crate::widgets::standard_card(tr!("Hardware"), None::<&str>)
            .min_height(200)
            .with_child(content)
            .into()
    }
}

impl From<VmHardwarePanel> for VNode {
    fn from(props: VmHardwarePanel) -> Self {
        let comp = VComp::new::<PveVmHardwarePanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
