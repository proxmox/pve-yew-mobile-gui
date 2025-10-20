use std::rc::Rc;

use pwt::widget::Column;
use pwt::{prelude::*, widget::form::Checkbox};

use pve_api_types::{StorageContent, StorageInfo};
use yew::virtual_dom::VComp;

use proxmox_yew_comp::form::pve::{qemu_image_format_selector, PveStorageSelector};
use proxmox_yew_comp::layout::mobile_form::label_field;
use proxmox_yew_comp::{PropertyEditDialog, PropertyEditorState};

#[derive(PartialEq, Properties, Clone)]
struct QemuMoveDiskPanel {
    node: Option<AttrValue>,
    state: PropertyEditorState,
}

enum Msg {
    StorageInfo(Option<StorageInfo>),
}

struct QemuMoveDiskPanelComp {
    storage_info: Option<StorageInfo>,
}

impl Component for QemuMoveDiskPanelComp {
    type Message = Msg;
    type Properties = QemuMoveDiskPanel;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { storage_info: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StorageInfo(info) => self.storage_info = info,
        }
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        // let state = &props.state;

        // fixme: detect available storage formats from self.storage_info
        let disable_format_selector = true;

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(label_field(
                tr!("Storage"),
                PveStorageSelector::new(props.node.clone())
                    .name("storage")
                    .required(true)
                    .autoselect(true)
                    .content_types(Some(vec![StorageContent::Images]))
                    .on_change(ctx.link().callback(Msg::StorageInfo))
                    .mobile(true),
                true,
            ))
            .with_child(label_field(
                tr!("Format"),
                qemu_image_format_selector().name("format"),
                !disable_format_selector,
            ))
            .with_child(label_field(
                tr!("Delete source"),
                Checkbox::new().name("delete"),
                !disable_format_selector,
            ))
            .into()
    }
}

pub fn qemu_move_disk_dialog(name: &str, node: Option<AttrValue>) -> PropertyEditDialog {
    let title = tr!("Move Disk");

    PropertyEditDialog::new(title.clone() + " (" + name + ")")
        .edit(false)
        .submit_text(title.clone())
        .renderer({
            let node = node.clone();
            move |state| {
                let props = QemuMoveDiskPanel {
                    state,
                    node: node.clone(),
                };
                VComp::new::<QemuMoveDiskPanelComp>(Rc::new(props), None).into()
            }
        })
        .submit_hook({
            let disk = name.to_string();
            move |state: PropertyEditorState| {
                let mut data = state.form_ctx.get_submit_data();
                data["disk"] = disk.clone().into();
                Ok(data)
            }
        })
}
