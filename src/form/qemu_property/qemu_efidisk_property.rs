use serde_json::Value;

use pwt::widget::form::{Checkbox, Combobox, Hidden};
use pwt::widget::Container;

use pwt::prelude::*;
use pwt::widget::Column;

use pve_api_types::{
    QemuConfigBios, QemuConfigEfidisk0, QemuConfigEfidisk0Efitype, StorageContent, StorageInfo,
};

const IMAGE_STORAGE: &'static str = "_storage_";
const STORAGE_INFO: &'static str = "_storage_info_";

const PRE_ENROLLED_KEYS: &'static str = "_pre-enrolled-keys";

use crate::form::{property_string_from_parts, PveStorageSelector};

use crate::widgets::{
    label_field, EditableProperty, PropertyEditorState, RenderPropertyInputPanelFn,
};

fn efidisk_input_panel(node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |state: PropertyEditorState| {
        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        let bios = serde_json::from_value::<Option<QemuConfigBios>>(state.record["bios"].clone());
        let bios_hint = match bios {
            Ok(Some(QemuConfigBios::Ovmf)) => false,
            _ => true,
        };

        let storage_info = state
            .form_ctx
            .read()
            .get_field_value(STORAGE_INFO)
            .unwrap_or(Value::Null);
        let _storage_info = serde_json::from_value::<Option<StorageInfo>>(storage_info).unwrap();

        // fixme: detect available storage formats
        let disable_format_selector = true;

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(Hidden::new().name(STORAGE_INFO).submit(false))
            .with_child(label_field(
                tr!("Storage"),
                PveStorageSelector::new(node.clone())
                    .name(IMAGE_STORAGE)
                    .submit(false)
                    .required(true)
                    .autoselect(true)
                    .content_types(Some(vec![StorageContent::Images]))
                    .on_change({
                        let form_ctx = state.form_ctx.clone();
                        move |info: Option<StorageInfo>| {
                            form_ctx
                                .write()
                                .set_field_value(STORAGE_INFO, serde_json::to_value(info).unwrap());
                        }
                    })
                    .mobile(true),
                true,
            ))
            .with_child(label_field(
                tr!("Format"),
                Combobox::from_key_value_pairs([
                    ("raw", tr!("Raw disk image") + " (raw)"),
                    ("qcow2", tr!("QEMU image format") + " (qcow2)"),
                    ("vmdk", tr!("VMware image format") + " (vmdk)"),
                ])
                .placeholder("raw")
                .name("_format"),
                !disable_format_selector,
            ))
            .with_child(label_field(
                tr!("Pre-Enroll keys"),
                Checkbox::new().name(PRE_ENROLLED_KEYS).submit(false),
                true,
            ))
            .with_optional_child(bios_hint.then(|| {
                hint(tr!(
                    "Warning: The VM currently does not uses 'OVMF (UEFI)' as BIOS."
                ))
            }))
            .into()
    })
}

pub fn qemu_efidisk_property(name: Option<String>, node: Option<AttrValue>) -> EditableProperty {
    let title = tr!("EFI Disk");
    EditableProperty::new(name.clone(), title)
        .render_input_panel(efidisk_input_panel(node.clone()))
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
