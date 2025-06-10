use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use proxmox_schema::ApiType;
use pwt::prelude::*;
use pwt::widget::{Card, Fa, List, ListTile};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::{PveQmIde, PveQmIdeMedia, QemuConfig};

use super::icon_list_tile;

#[derive(Clone, PartialEq, Properties)]
pub struct VmConfigPanel {
    vmid: u64,
    node: AttrValue,
}

impl VmConfigPanel {
    pub fn new(node: impl Into<AttrValue>, vmid: u64) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

fn get_config_url(node: &str, vmid: u64) -> String {
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

pub struct PveVmConfigPanel {
    data: Result<QemuConfig, String>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
}

impl PveVmConfigPanel {
    fn view_config(&self, _ctx: &Context<Self>, data: &QemuConfig) -> Html {
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

        Card::new()
            .border(true)
            .padding(0)
            .class("pwt-flex-none pwt-overflow-hidden")
            .with_child(html! {<div class="pwt-p-2 pwt-font-size-title-large">{"Hardware"}</div>})
            .with_child(
                List::new(list.len() as u64, move |pos| list[pos as usize].clone())
                    .grid_template_columns("auto 1fr auto"),
            )
            .into()
    }
}

impl Component for PveVmConfigPanel {
    type Message = Msg;
    type Properties = VmConfigPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: Err(format!("no data loaded")),
            reload_timeout: None,
            load_guard: None,
        }
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
                self.data = result.map_err(|err| err.to_string());
                let link = ctx.link().clone();
                self.reload_timeout = Some(Timeout::new(3000, move || {
                    link.send_message(Msg::Load);
                }));
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.data {
            Ok(data) => self.view_config(ctx, data),
            Err(err) => Card::new()
                .border(true)
                .class("pwt-card-bordered")
                .with_child(pwt::widget::error_message(err))
                .into(),
        }
    }
}

impl From<VmConfigPanel> for VNode {
    fn from(props: VmConfigPanel) -> Self {
        let comp = VComp::new::<PveVmConfigPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
