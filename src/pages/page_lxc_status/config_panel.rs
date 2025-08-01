use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use serde_json::json;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::form::{Checkbox, Form, FormContext};
use pwt::widget::{List, ListTile, Progress};

use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, http_put, percent_encoding::percent_encode_component};

use pve_api_types::LxcConfig;

use crate::widgets::form_list_tile;

#[derive(Clone, PartialEq, Properties)]
pub struct LxcConfigPanel {
    vmid: u32,
    node: AttrValue,
}

impl LxcConfigPanel {
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
    StoreBoolConfig(&'static str, bool),
    StoreResult(Result<(), Error>),
}

pub struct PveLxcConfigPanel {
    data: Option<Result<LxcConfig, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    store_guard: Option<AsyncAbortGuard>,
    form_context: FormContext,
}

impl PveLxcConfigPanel {
    fn changeable_config_bool(
        &self,
        ctx: &Context<Self>,
        title: impl Into<AttrValue>,
        name: &'static str,
        default: bool,
    ) -> ListTile {
        let switch = Checkbox::new()
            .switch(true)
            .name(name)
            .default(default)
            .on_input(
                ctx.link()
                    .callback(move |value| Msg::StoreBoolConfig(name, value)),
            );

        form_list_tile(title, None::<&str>, Some(switch.into()))
    }

    fn view_config(&self, ctx: &Context<Self>, data: &LxcConfig) -> Html {
        let props = ctx.props();

        let mut list: Vec<ListTile> = Vec::new();

        list.push(self.changeable_config_bool(ctx, tr!("Start on boot"), "onboot", false));
        list.push(self.changeable_config_bool(ctx, tr!("Protection"), "protection", false));
        list.push(self.changeable_config_bool(ctx, tr!("Console"), "console", true));

        list.push(form_list_tile(
            tr!("Name"),
            data.hostname
                .as_ref()
                .map(String::from)
                .unwrap_or(format!("CT {}", props.vmid)),
            None,
        ));

        list.push(form_list_tile(
            tr!("Start/Shutdown order"),
            data.startup
                .as_ref()
                .map(String::from)
                .unwrap_or(tr!("Default (any)")),
            None,
        ));

        list.push(form_list_tile(
            tr!("OS Type"),
            data.ostype
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or(tr!("Other")),
            None,
        ));

        list.push(form_list_tile(
            tr!("Architecture"),
            data.arch
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or(String::from("-")),
            None,
        ));

        list.push(form_list_tile(
            tr!("TTY count"),
            data.tty.unwrap_or(2).to_string(),
            None,
        ));

        list.push(form_list_tile(
            tr!("Console mode"),
            data.cmode
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or(String::from("tty")),
            None,
        ));

        list.push(form_list_tile(
            tr!("Unpriviledged"),
            data.unprivileged.unwrap_or(false).to_string(),
            None,
        ));

        list.push(form_list_tile(
            tr!("Features"),
            data.features
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or(tr!("none")),
            None,
        ));

        Form::new()
            .class(pwt::css::FlexFit)
            .form_context(self.form_context.clone())
            .with_child(
                List::new(list.len() as u64, move |pos| list[pos as usize].clone())
                    .class(pwt::css::FlexFit)
                    .grid_template_columns("1fr auto"),
            )
            .into()
    }
}

impl Component for PveLxcConfigPanel {
    type Message = Msg;
    type Properties = LxcConfigPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
            reload_timeout: None,
            load_guard: None,
            store_guard: None,
            form_context: FormContext::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Load => {
                self.reload_timeout = None;
                let link = ctx.link().clone();
                let url = get_config_url(&props.node, props.vmid);
                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&url, None).await;
                    link.send_message(Msg::LoadResult(result));
                }));
            }
            Msg::LoadResult(result) => {
                self.data = Some(result.map_err(|err| err.to_string()));
                if let Some(Ok(data)) = &self.data {
                    self.form_context
                        .load_form(serde_json::to_value(data).unwrap());
                }
                let link = ctx.link().clone();
                self.reload_timeout = Some(Timeout::new(3000, move || {
                    link.send_message(Msg::Load);
                }));
            }
            Msg::StoreBoolConfig(name, value) => {
                let link = ctx.link().clone();
                let url = get_config_url(&props.node, props.vmid);
                let mut param = json!({});
                param[name] = value.into();
                self.store_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_put(&url, Some(param)).await;
                    link.send_message(Msg::StoreResult(result));
                }));
            }
            Msg::StoreResult(result) => {
                if self.reload_timeout.is_some() {
                    ctx.link().send_message(Msg::Load);
                }
                if let Err(err) = result {
                    crate::show_failed_command_error(ctx.link(), err);
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.data {
            Some(Ok(data)) => self.view_config(ctx, data),
            Some(Err(err)) => pwt::widget::error_message(err).into(),
            None => Progress::new().class("pwt-delay-visibility").into(),
        }
    }
}

impl From<LxcConfigPanel> for VNode {
    fn from(props: LxcConfigPanel) -> Self {
        let comp = VComp::new::<PveLxcConfigPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
