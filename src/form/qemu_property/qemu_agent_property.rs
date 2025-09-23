use pve_api_types::QemuConfigAgent;

use pwt::prelude::*;
use pwt::widget::form::{Checkbox, Combobox};
use pwt::widget::Column;
use pwt::widget::{form::FormContext, Container};

use crate::form::{property_string_load_hook, property_string_submit_hook, pspn};
use crate::widgets::{EditableProperty, RenderPropertyInputPanelFn};

fn input_panel(name: String) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, _record| {
        let advanced = form_ctx.get_show_advanced();
        let enabled = form_ctx.read().get_field_checked(pspn(&name, "enabled"));
        let ffob_enabled = form_ctx
            .read()
            .get_field_checked(pspn(&name, "freeze-fs-on-backup"));

        let warning = |msg: String| {
            Container::new()
                .class("pwt-color-warning")
                .padding(1)
                .with_child(msg)
        };

        Column::new()
                        .class(pwt::css::FlexFit)
                        .with_child(
                            Checkbox::new()
                                .name(pspn(&name, "enabled"))
                                .box_label(tr!("Use QEMU Guest Agent")),
                        )
                        .with_child(
                            Checkbox::new()
                                .name(pspn(&name, "fstrim_cloned_disks"))
                                .box_label(tr!("Run guest-trim after a disk move or VM migration"))
                                .disabled(!enabled),
                        )
                        .with_child(
                            Checkbox::new()
                                .name(pspn(&name, "freeze-fs-on-backup"))
                                .box_label(tr!(
                                    "Freeze/thaw guest filesystems on backup for consistency"
                                ))
                                .disabled(!enabled),
                        )
                        .with_child(
                            crate::widgets::label_field(
                                tr!("Type"),
                                Combobox::from_key_value_pairs([
                                        ("virtio", "VirtIO"),
                                        ("isa", "ISA"),
                                ])
                                    .name(pspn(&name, "type"))
                                    .placeholder(tr!("Default") + " (VirtIO)")
                            ).class((!advanced).then(|| pwt::css::Display::None))
                            .padding_top(2)
                            .padding_bottom(1)
                        )
                        .with_optional_child((!ffob_enabled).then(|| warning(tr!(
                            "Freeze/thaw for guest filesystems disabled. This can lead to inconsistent disk backups."
                        ))))
                        .with_optional_child(enabled.then(|| warning(tr!(
                            "Make sure the QEMU Guest Agent is installed in the VM"
                        ))))
                        .into()
    })
}

pub fn qemu_agent_property() -> EditableProperty {
    let name = String::from("agent");
    EditableProperty::new(name.clone(), tr!("QEMU Guest Agent"))
        .advanced_checkbox(true)
        .required(true)
        .placeholder(format!("{} ({})", tr!("Default"), tr!("Disabled")))
        .render_input_panel(input_panel(name.clone()))
        .load_hook(property_string_load_hook::<QemuConfigAgent>(&name))
        .submit_hook(property_string_submit_hook::<QemuConfigAgent>(&name, true))
}
