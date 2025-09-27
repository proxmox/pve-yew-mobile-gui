use indexmap::IndexMap;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Combobox};
use pwt::widget::Column;

use crate::widgets::{EditableProperty, PropertyEditorState};

pub fn qemu_scsihw_property() -> EditableProperty {
    const NAME: &'static str = "scsihw";
    let mut items = IndexMap::new();
    items.extend([
        ("lsi", "LSI 53C895A"),
        ("lsi53c810", "LSI 53C810"),
        ("megasas", "MegaRAID SAS 8708EM2"),
        ("virtio-scsi-pci", "VirtIO SCSI"),
        ("virtio-scsi-single", "VirtIO SCSI single"),
        ("pvscsi", "VMware PVSCSI"),
    ]);
    let placeholder = tr!("Default") + " (LSI 53C895A)";

    EditableProperty::new(NAME, "scsihw")
        .placeholder(placeholder.clone())
        .renderer({
            let items = items.clone();
            move |_, v, _| match items.get(v.as_str().unwrap_or("")) {
                Some(descr) => descr.into(),
                None => v.into(),
            }
        })
        .render_input_panel(move |_| {
            Column::new()
                .class(pwt::css::FlexFit)
                .gap(2)
                .padding_bottom(1) // avoid scrollbar ?!
                .with_child(
                    Combobox::from_key_value_pairs(items.clone())
                        .name(NAME)
                        .submit_empty(true)
                        .placeholder(tr!("Default") + " (LSI 53C895A)"),
                )
                .into()
        })
        .submit_hook({
            move |state: PropertyEditorState| {
                let record = state.get_submit_data();
                let record = delete_empty_values(&record, &[NAME], false);
                Ok(record)
            }
        })
}
