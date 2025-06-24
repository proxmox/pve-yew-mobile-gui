use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Card, List, ListTile};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::QemuConfig;

use crate::widgets::standard_list_tile;

#[derive(Clone, PartialEq, Properties)]
pub struct VmConfigPanel {
    vmid: u32,
    node: AttrValue,
}

impl VmConfigPanel {
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
        list.push(standard_list_tile(
            tr!("Name"),
            data.name.as_deref().unwrap_or("-").to_string(),
            None,
            None,
        ));

        List::new(list.len() as u64, move |pos| list[pos as usize].clone())
            .grid_template_columns("auto 1fr auto")
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
            Err(err) => pwt::widget::error_message(err).into(),
        }
    }
}

impl From<VmConfigPanel> for VNode {
    fn from(props: VmConfigPanel) -> Self {
        let comp = VComp::new::<PveVmConfigPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
