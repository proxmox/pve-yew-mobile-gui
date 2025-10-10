use std::collections::HashSet;
use std::rc::Rc;

use anyhow::Error;
use proxmox_yew_comp::http_get;
use serde_json::Value;
use yew::virtual_dom::VComp;

use pve_api_types::ClusterResource;

use pwt::widget::Column;
use pwt::{prelude::*, AsyncAbortGuard};

use crate::form::extract_used_devices;
use crate::form::{PveGuestSelector, PveGuestType, QemuControllerSelector};
use crate::widgets::{label_field, EditDialog, PropertyEditorState};

#[derive(PartialEq, Properties, Clone)]
struct QemuReassignDiskPanel {
    node: Option<AttrValue>,
    state: PropertyEditorState,
}

enum Msg {
    Target(Option<ClusterResource>),
    LoadResult(Result<Value, Error>),
}

struct QemuReassignDiskPanelComp {
    target: Option<ClusterResource>,
    load_guard: Option<AsyncAbortGuard>,
    used_devices: Option<HashSet<String>>,
}

impl Component for QemuReassignDiskPanelComp {
    type Message = Msg;
    type Properties = QemuReassignDiskPanel;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            target: None,
            load_guard: None,
            used_devices: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Target(target) => {
                self.target = target;
                if let Some(ClusterResource {
                    node: Some(node),
                    vmid: Some(vmid),
                    ..
                }) = &self.target
                {
                    let url = super::QemuHardwarePanel::new(node.clone(), *vmid).editor_url();
                    let link = ctx.link().clone();
                    self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                        let result = http_get(&url, None).await;
                        link.send_message(Msg::LoadResult(result));
                    }));
                }
            }
            Msg::LoadResult(result) => {
                match result {
                    Ok(data) => self.used_devices = Some(extract_used_devices(&data)),
                    Err(err) => {
                        log::error!("QemuReassignDiskPanel: load target config failed - {err}");
                        self.used_devices = None;
                    }
                };
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        //let props = ctx.props();
        // let state = &props.state;
        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(label_field(
                tr!("Target Guest"),
                PveGuestSelector::new()
                    .name("target-vmid")
                    .required(true)
                    .guest_type(PveGuestType::Qemu)
                    .on_change(ctx.link().callback(Msg::Target))
                    .mobile(true),
                true,
            ))
            .with_child(label_field(
                tr!("Bus/Device"),
                QemuControllerSelector::new()
                    .name("target-disk")
                    .exclude_devices(self.used_devices.clone()),
                true,
            ))
            .into()
    }
}

pub fn qemu_reassign_disk_dialog(name: &str, node: Option<AttrValue>) -> EditDialog {
    let title = tr!("Reassign Disk");

    EditDialog::new(title.clone() + " (" + name + ")")
        .edit(false)
        .submit_text(title.clone())
        .submit_hook({
            let disk = name.to_string();
            move |state: PropertyEditorState| {
                let mut data = state.form_ctx.get_submit_data();

                data["disk"] = disk.clone().into();
                Ok(data)
            }
        })
        .renderer({
            let node = node.clone();
            move |state| {
                let props = QemuReassignDiskPanel {
                    state,
                    node: node.clone(),
                };
                VComp::new::<QemuReassignDiskPanelComp>(Rc::new(props), None).into()
            }
        })
}
