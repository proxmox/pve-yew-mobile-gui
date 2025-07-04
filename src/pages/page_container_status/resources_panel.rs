use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, Fa, List, ListTile, Progress};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::LxcConfig;

use crate::widgets::icon_list_tile;

#[derive(Clone, PartialEq, Properties)]
pub struct ContainerResourcesPanel {
    vmid: u32,
    node: AttrValue,
}

impl ContainerResourcesPanel {
    pub fn new(node: impl Into<AttrValue>, vmid: u32) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

fn get_config_url(node: &str, vmid: u32) -> String {
    format!(
        "/nodes/{}/lxc/{}/config",
        percent_encode_component(node),
        vmid
    )
}

pub enum Msg {
    Load,
    LoadResult(Result<LxcConfig, Error>),
}

pub struct PveContainerResourcesPanel {
    data: Option<Result<LxcConfig, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
}

impl PveContainerResourcesPanel {
    fn resource_info(&self, _ctx: &Context<Self>, data: &LxcConfig) -> Html {
        let mut list: Vec<ListTile> = Vec::new();
        list.push(icon_list_tile(
            Fa::new("memory"),
            data.memory.unwrap_or(512).to_string() + " MB",
            tr!("Memory"),
            None,
        ));
        list.push(icon_list_tile(
            Fa::new("retweet"),
            data.swap.unwrap_or(0).to_string() + " MB",
            tr!("Swap"),
            None,
        ));

        list.push(icon_list_tile(
            Fa::new("cpu"),
            data.cores.unwrap_or(1).to_string(),
            tr!("Cores"),
            None,
        ));

        list.push(icon_list_tile(
            Fa::new("hdd-o"),
            data.rootfs.as_deref().unwrap_or("-").to_string(),
            tr!("Root Filesystem"),
            None,
        ));

        crate::widgets::standard_card(tr!("Resources"), None::<&str>)
            .with_child(
                List::new(list.len() as u64, move |pos| list[pos as usize].clone())
                    .grid_template_columns("auto 1fr auto"),
            )
            .into()
    }

    fn network_info(&self, _ctx: &Context<Self>, data: &LxcConfig) -> Html {
        let mut list: Vec<ListTile> = Vec::new();

        for (n, net_config) in &data.net {
            list.push(icon_list_tile(
                Fa::new("exchange"),
                net_config.to_string(),
                tr!("Network Device") + &format!(" (net{n})"),
                None,
            ));
        }

        crate::widgets::standard_card(tr!("Network"), None::<&str>)
            .with_child(
                List::new(list.len() as u64, move |pos| list[pos as usize].clone())
                    .grid_template_columns("auto 1fr auto"),
            )
            .into()
    }

    fn dns_info(&self, _ctx: &Context<Self>, data: &LxcConfig) -> Html {
        let mut list: Vec<ListTile> = Vec::new();

        list.push(icon_list_tile(
            Fa::new("globe"),
            data.searchdomain
                .clone()
                .unwrap_or(tr!("Use host settings")),
            tr!("DNS Domain"),
            None,
        ));

        list.push(icon_list_tile(
            Fa::new("search"),
            data.nameserver.clone().unwrap_or(tr!("Use host settings")),
            tr!("Nameserver"),
            None,
        ));

        crate::widgets::standard_card(tr!("DNS"), None::<&str>)
            .with_child(
                List::new(list.len() as u64, move |pos| list[pos as usize].clone())
                    .grid_template_columns("auto 1fr auto"),
            )
            .into()
    }
}

impl Component for PveContainerResourcesPanel {
    type Message = Msg;
    type Properties = ContainerResourcesPanel;

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
        match &self.data {
            Some(Ok(data)) => Column::new()
                .gap(2)
                .with_child(self.resource_info(ctx, data))
                .with_child(self.network_info(ctx, data))
                .with_child(self.dns_info(ctx, data))
                .into(),
            Some(Err(err)) => pwt::widget::error_message(err).into(),
            None => Progress::new().class("pwt-delay-visibility").into(),
        }
    }
}

impl From<ContainerResourcesPanel> for VNode {
    fn from(props: ContainerResourcesPanel) -> Self {
        let comp = VComp::new::<PveContainerResourcesPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
