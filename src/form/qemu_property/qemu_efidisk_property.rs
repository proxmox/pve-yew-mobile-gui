use std::rc::Rc;

use pwt::widget::form::Checkbox;
use pwt::widget::Container;

use pwt::prelude::*;
use pwt::widget::Column;

use pve_api_types::{
    QemuConfigBios, QemuConfigEfidisk0, QemuConfigEfidisk0Efitype, StorageContent, StorageInfo,
};
use yew::virtual_dom::VComp;

const IMAGE_STORAGE: &'static str = "_storage_";

use crate::form::{property_string_from_parts, qemu_image_format_selector, PveStorageSelector};

use crate::widgets::{label_field, EditableProperty, PropertyEditorState};

#[derive(PartialEq, Properties)]
struct QemuEfidiskPanel {
    node: Option<AttrValue>,
    state: PropertyEditorState,
}

enum Msg {
    StorageInfo(Option<StorageInfo>),
}
struct QemuEfidiskPanelComp {
    storage_info: Option<StorageInfo>,
}

impl Component for QemuEfidiskPanelComp {
    type Message = Msg;
    type Properties = QemuEfidiskPanel;

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
        let state = &props.state;

        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        let bios = serde_json::from_value::<Option<QemuConfigBios>>(state.record["bios"].clone());
        let bios_hint = match bios {
            Ok(Some(QemuConfigBios::Ovmf)) => false,
            _ => true,
        };

        // fixme: detect available storage formats from self.storage_info
        let disable_format_selector = true;

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(label_field(
                tr!("Storage"),
                PveStorageSelector::new(props.node.clone())
                    .name(IMAGE_STORAGE)
                    .submit(false)
                    .required(true)
                    .autoselect(true)
                    .content_types(Some(vec![StorageContent::Images]))
                    .on_change(ctx.link().callback(Msg::StorageInfo))
                    .mobile(true),
                true,
            ))
            .with_child(label_field(
                tr!("Format"),
                qemu_image_format_selector().name("_format"),
                !disable_format_selector,
            ))
            .with_child(label_field(
                tr!("Pre-Enroll keys"),
                Checkbox::new().name("_pre-enrolled-keys").submit(false),
                true,
            ))
            .with_optional_child(bios_hint.then(|| {
                hint(tr!(
                    "Warning: The VM currently does not uses 'OVMF (UEFI)' as BIOS."
                ))
            }))
            .into()
    }
}

pub fn qemu_efidisk_property(name: Option<AttrValue>, node: Option<AttrValue>) -> EditableProperty {
    let title = tr!("EFI Disk");
    EditableProperty::new(name.clone(), title)
        .render_input_panel(move |state| {
            let props = QemuEfidiskPanel {
                state,
                node: node.clone(),
            };
            VComp::new::<QemuEfidiskPanelComp>(Rc::new(props), None).into()
        })
        .submit_hook(move |state: PropertyEditorState| {
            let form_ctx = &state.form_ctx;
            let mut data = form_ctx.get_submit_data();

            let storage = form_ctx.read().get_field_text(IMAGE_STORAGE);

            // we use 1 here, because for efi the size gets overridden from the backend
            data["_file"] = format!("{}:1", storage).into();
            // always default to newer 4m type with secure boot support, if we're
            // adding a new EFI disk there can't be any old state anyway
            data["_efitype"] = QemuConfigEfidisk0Efitype::Mb4.to_string().into();

            property_string_from_parts::<QemuConfigEfidisk0>(&mut data, "efidisk0", true)?;
            Ok(data)
        })
}
