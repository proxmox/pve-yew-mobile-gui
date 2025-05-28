use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use proxmox_schema::ApiType;
use pwt::prelude::*;
use pwt::widget::{Card, Column, Fa, ListTile};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::{PveQmIde, PveQmIdeMedia, QemuConfig};

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

fn create_config_tile(icon: &str, title: &str, subtitle: &str) -> ListTile {
    let icon: Html = Fa::new(icon).fixed_width().large_2x().into();

    ListTile::new()
        .class("pwt-d-flex")
        .class("pwt-gap-2")
        .class("pwt-scheme-surface")
        .border_top(true)
        .with_child(icon)
        .with_child(
            Column::new()
                .gap(1)
                .with_child(html! {<div class="pwt-font-size-title-medium">{title}</div>})
                .with_child(html! {<div class="pwt-font-size-title-small">{subtitle}</div>}),
        )
}

impl PveVmConfigPanel {
    fn view_config(&self, _ctx: &Context<Self>, data: &QemuConfig) -> Html {
        let mut card = Card::new()
            .border(true)
            .padding(0)
            .class("pwt-flex-none pwt-overflow-hidden")
            .with_child(html! {<div class="pwt-p-2 pwt-font-size-title-large">{"Hardware"}</div>})
            .with_child(create_config_tile(
                "memory",
                data.memory.as_deref().unwrap_or("-"),
                "Memory",
            ))
            .with_child(create_config_tile(
                "cpu",
                &processor_text(data),
                "Processor",
            ))
            .with_child(create_config_tile(
                "microchip",
                &data
                    .bios
                    .map(|b| b.to_string())
                    .unwrap_or(String::from("Default (SeaBIOS)")),
                "Bios",
            ))
            .with_child(create_config_tile(
                "gears",
                &data.machine.as_deref().unwrap_or("Default (i440fx)"),
                "Machine Type",
            ));

        for (n, disk_config) in &data.ide {
            if let Ok(config) = PveQmIde::API_SCHEMA.parse_property_string(disk_config) {
                if let Ok(config) = serde_json::from_value::<PveQmIde>(config) {
                    if config.media == Some(PveQmIdeMedia::Cdrom) {
                        card.add_child(create_config_tile(
                            "cdrom",
                            disk_config,
                            &format!("CD/DVD Drive (ide{n})"),
                        ));
                    } else {
                        card.add_child(create_config_tile(
                            "hdd-o",
                            disk_config,
                            &format!("Hard Disk (ide{n})"),
                        ));
                    }
                }
            }
        }

        for (n, disk_config) in &data.scsi {
            card.add_child(create_config_tile(
                "hdd-o",
                disk_config,
                &format!("Hard Disk (scsi{n})"),
            ));
        }

        for (n, net_config) in &data.net {
            card.add_child(create_config_tile(
                "exchange",
                net_config,
                &format!("Network Device (new{n})"),
            ));
        }

        card.into()
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
