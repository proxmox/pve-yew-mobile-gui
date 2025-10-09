use pwt::prelude::*;
use pwt::widget::Column;

use crate::form::{PveGuestSelector, QemuControllerSelector};
use crate::widgets::EditDialog;
use crate::widgets::{label_field, PropertyEditorState};

pub fn qemu_reassign_disk_dialog(name: &str, _node: Option<AttrValue>) -> EditDialog {
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
        .renderer(|_| {
            Column::new()
                .class(pwt::css::FlexFit)
                .gap(2)
                .with_child(label_field(
                    tr!("Target Guest"),
                    PveGuestSelector::new().name("target-vmid"),
                    true,
                ))
                .with_child(label_field(
                    tr!("Bus/Device"),
                    QemuControllerSelector::new().name("target-disk"),
                    //.exclude_devices(used_devices),
                    true,
                ))
                .into()
        })
}
