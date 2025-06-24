use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use serde_json::json;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::{SnackBar, SnackBarContextExt};
use pwt::widget::form::{Checkbox, Form, FormContext};
use pwt::widget::{List, ListTile};

use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, http_post, percent_encoding::percent_encode_component};

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
    StoreBoolConfig(&'static str, bool),
    StoreResult(Result<(), Error>),
}

pub struct PveVmConfigPanel {
    data: Result<QemuConfig, String>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    store_guard: Option<AsyncAbortGuard>,
    form_context: FormContext,
}

impl PveVmConfigPanel {
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

        standard_list_tile(title.into(), None::<&str>, None, Some(switch.into()))
    }

    fn view_config(&self, ctx: &Context<Self>, data: &QemuConfig) -> Html {
        let props = ctx.props();

        let mut list: Vec<ListTile> = Vec::new();

        list.push(self.changeable_config_bool(ctx, tr!("Start on boot"), "onboot", false));
        list.push(self.changeable_config_bool(ctx, tr!("Use tablet for pointer"), "tablet", true));
        list.push(self.changeable_config_bool(ctx, tr!("ACPI support"), "acpi", true));
        list.push(self.changeable_config_bool(
            ctx,
            tr!("KVM hardware virtualization"),
            "kvm",
            true,
        ));
        list.push(self.changeable_config_bool(ctx, tr!("Freeze CPU on startup"), "freeze", false));
        list.push(self.changeable_config_bool(
            ctx,
            tr!("Use local time for RTC"),
            "localtime",
            false,
        ));
        list.push(self.changeable_config_bool(ctx, tr!("Protection"), "protection", false));

        list.push(standard_list_tile(
            tr!("Name"),
            data.name
                .as_ref()
                .map(String::from)
                .unwrap_or(format!("VM {}", props.vmid)),
            None,
            None,
        ));

        list.push(standard_list_tile(
            tr!("Start/Shutdown order"),
            data.startup
                .as_ref()
                .map(String::from)
                .unwrap_or(tr!("Default (any)")),
            None,
            None,
        ));

        list.push(standard_list_tile(
            tr!("OS Type"),
            data.ostype
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or(tr!("Other")),
            None,
            None,
        ));

        list.push(standard_list_tile(
            tr!("Boot Device"),
            data.boot
                .as_ref()
                .map(String::from)
                .unwrap_or(tr!("Disk, Network, USB")),
            None,
            None,
        ));

        list.push(standard_list_tile(
            tr!("Hotplug"),
            data.hotplug
                .as_ref()
                .map(String::from)
                .unwrap_or(String::from("disk,network,usb")),
            None,
            None,
        ));

        list.push(standard_list_tile(
            tr!("RTC start date"),
            data.startdate
                .as_ref()
                .map(String::from)
                .unwrap_or(String::from("now")),
            None,
            None,
        ));

        list.push(standard_list_tile(
            tr!("SMBIOS settings (type1)"),
            data.smbios1
                .as_ref()
                .map(String::from)
                .unwrap_or(String::from("-")),
            None,
            None,
        ));

        list.push(standard_list_tile(
            tr!("QEMU Guest Agent"),
            data.smbios1
                .as_ref()
                .map(String::from)
                .unwrap_or(tr!("Default (disabled)")),
            None,
            None,
        ));

        list.push(standard_list_tile(
            tr!("Spice Enhancements"),
            data.spice_enhancements
                .as_ref()
                .map(String::from)
                .unwrap_or(tr!("No enhancements")),
            None,
            None,
        ));

        list.push(standard_list_tile(
            tr!("VM State Storage"),
            data.vmstatestorage
                .as_ref()
                .map(String::from)
                .unwrap_or(String::from("1 (autogenerated)")),
            None,
            None,
        ));

        Form::new()
            .class(pwt::css::FlexFit)
            .form_context(self.form_context.clone())
            .with_child(
                List::new(list.len() as u64, move |pos| list[pos as usize].clone())
                    .class(pwt::css::FlexFit)
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
                self.data = result.map_err(|err| err.to_string());
                if let Ok(data) = &self.data {
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
                let url = get_config_url(&props.node, 10);
                let mut param = json!({});
                param[name] = value.into();
                self.store_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_post(&url, Some(param)).await;
                    link.send_message(Msg::StoreResult(result));
                }));
            }
            Msg::StoreResult(result) => {
                if self.reload_timeout.is_some() {
                    ctx.link().send_message(Msg::Load);
                }
                if let Err(err) = result {
                    ctx.link()
                        .show_snackbar(SnackBar::new().message(format!("Update failed: {err}")));
                }
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
