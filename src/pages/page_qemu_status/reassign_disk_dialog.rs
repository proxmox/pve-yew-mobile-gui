use std::rc::Rc;

use yew::virtual_dom::VComp;

use pve_api_types::ClusterResource;

use pwt::prelude::*;
use pwt::widget::Column;

use crate::form::{PveGuestSelector, PveGuestType, QemuControllerSelector};
use crate::widgets::{label_field, EditDialog, PropertyEditorState};

#[derive(PartialEq, Properties, Clone)]
struct QemuReassignDiskPanel {
    node: Option<AttrValue>,
    state: PropertyEditorState,
}

enum Msg {
    Target(Option<ClusterResource>),
}

struct QemuReassignDiskPanelComp {
    target: Option<ClusterResource>,
}

impl Component for QemuReassignDiskPanelComp {
    type Message = Msg;
    type Properties = QemuReassignDiskPanel;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { target: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Target(target) => self.target = target,
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
                QemuControllerSelector::new().name("target-disk"),
                //.exclude_devices(used_devices),
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
