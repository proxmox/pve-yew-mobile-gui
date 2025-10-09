use pwt::widget::form::Combobox;

use pwt::prelude::*;
use pwt::widget::Column;

use pve_api_types::{QemuConfigTpmstate0, StorageContent};

const IMAGE_STORAGE: &'static str = "_storage_";

use crate::form::{property_string_from_parts, PveStorageSelector};

use crate::widgets::{
    label_field, EditableProperty, PropertyEditorState, RenderPropertyInputPanelFn,
};

fn input_panel(node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_state: PropertyEditorState| {
        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(label_field(
                tr!("Storage"),
                PveStorageSelector::new(node.clone())
                    .name(IMAGE_STORAGE)
                    .submit(false)
                    .required(true)
                    .autoselect(true)
                    .content_types(Some(vec![StorageContent::Images]))
                    .mobile(true),
                true,
            ))
            .with_child(label_field(
                tr!("Version"),
                Combobox::new()
                    .with_item("v1.2")
                    .with_item("v2.0")
                    .default("v2.0")
                    .name("_version")
                    .required(true),
                true,
            ))
            .into()
    })
}

pub fn qemu_tpmstate_property(name: Option<String>, node: Option<AttrValue>) -> EditableProperty {
    let title = tr!("TPM State");
    EditableProperty::new(name.clone(), title)
        .render_input_panel(input_panel(node))
        .submit_hook(move |state: PropertyEditorState| {
            let form_ctx = &state.form_ctx;
            let mut data = form_ctx.get_submit_data();

            let storage = form_ctx.read().get_field_text(IMAGE_STORAGE);

            // we use 1 here, because for tpmstate the size gets overridden from the backend
            data["_file"] = format!("{}:1", storage).into();

            property_string_from_parts::<QemuConfigTpmstate0>(&mut data, "tpmstate0", true)?;
            Ok(data)
        })
}
