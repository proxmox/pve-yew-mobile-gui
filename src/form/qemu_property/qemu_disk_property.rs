use pwt::{prelude::*, widget::Column};

use crate::widgets::{EditableProperty, RenderPropertyInputPanelFn};

fn input_panel(node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_| {
        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child("TEST")
            .into()
    })
}

pub fn qemu_disk_property(name: Option<String>, node: Option<AttrValue>) -> EditableProperty {
    let mut title = tr!("Hard Disk");
    if let Some(name) = name.as_deref() {
        title = title + " (" + name + ")";
    }
    EditableProperty::new(
        name.as_ref().map(|s| s.clone()).unwrap_or(String::new()),
        title,
    )
    .render_input_panel(input_panel(node.clone()))
}

fn cdrom_input_panel(node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_| {
        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child("CDROM TEST")
            .into()
    })
}

pub fn qemu_cdrom_property(name: Option<String>, node: Option<AttrValue>) -> EditableProperty {
    let mut title = tr!("CD/DVD Drive");
    if let Some(name) = name.as_deref() {
        title = title + " (" + name + ")";
    }
    EditableProperty::new(
        name.as_ref().map(|s| s.clone()).unwrap_or(String::new()),
        title,
    )
    .render_input_panel(cdrom_input_panel(node.clone()))
}
