use anyhow::bail;
use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::Number;
use pwt::widget::Column;

use proxmox_yew_comp::layout::mobile_form::label_field;
use proxmox_yew_comp::{PropertyEditDialog, PropertyEditorState};

pub fn qemu_resize_disk_dialog(name: &str, _node: Option<AttrValue>) -> PropertyEditDialog {
    let title = tr!("Resize Disk");

    PropertyEditDialog::new(title.clone() + " (" + name + ")")
        .edit(false)
        .submit_text(title.clone())
        .submit_hook({
            let disk = name.to_string();
            move |state: PropertyEditorState| {
                let mut data = state.form_ctx.get_submit_data(); // get digest
                let incr = match state
                    .form_ctx
                    .read()
                    .get_last_valid_value("_size_increment_")
                {
                    Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
                    _ => bail!("invalid size increase - internal error"),
                };
                data["disk"] = disk.clone().into();
                data["size"] = format!("+{incr}G").into();
                Ok(data)
            }
        })
        .renderer(|_| {
            Column::new()
                .class(pwt::css::FlexFit)
                .gap(2)
                .with_child(label_field(
                    tr!("Size Increment") + " (" + &tr!("GiB") + ")",
                    Number::<f64>::new()
                        .name("_size_increment_")
                        .default(0.0)
                        .min(0.0)
                        .max(128.0 * 1024.0)
                        .submit(false),
                    true,
                ))
                .into()
        })
}
