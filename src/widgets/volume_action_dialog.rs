use std::rc::Rc;

use proxmox_human_byte::HumanByte;
use proxmox_yew_comp::{default_confirm_remove_message, ConfirmButton};
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::{MaterialAppScopeExt, SideDialog};
use pwt::widget::{Button, Column, Container, Row};

use crate::StorageEntry;

pub fn show_volume_actions<COMP: Component>(
    scope: yew::html::Scope<COMP>,
    node: impl Into<AttrValue>,
    storage: impl Into<AttrValue>,
    item: StorageEntry,
) {
    let controller = scope.page_controller().unwrap();

    log::info!("SHOW ACTIONS");
    controller.show_side_dialog(
        SideDialog::new()
            .location(pwt::touch::SideDialogLocation::Bottom)
            .with_child(VolumeActionDialog::new(node, storage, item)),
    );
}
#[derive(Clone, PartialEq, Properties)]
pub struct VolumeActionDialog {
    storage: AttrValue,
    node: AttrValue,
    item: StorageEntry,
}

impl VolumeActionDialog {
    pub fn new(
        node: impl Into<AttrValue>,
        storage: impl Into<AttrValue>,
        item: StorageEntry,
    ) -> Self {
        yew::props!(Self {
            node: node.into(),
            storage: storage.into(),
            item,
        })
    }
}

pub enum Msg {
    Remove,
    Cancel,
}

pub struct PveVolumeActionDialog {}

impl PveVolumeActionDialog {
    fn show_safe_remove_dialog(link: yew::html::Scope<Self>, volid: &str) {
        let content = Column::new()
            .class(pwt::css::FlexFit)
            .class(pwt::css::AlignItems::Center)
            .padding(2)
            .gap(2)
            .with_child(
                Container::new()
                    .class("pwt-font-size-title-large")
                    .with_child(tr!("Attention")),
            )
            .with_child(
                Container::new()
                    .class("pwt-font-size-title-medium")
                    .with_child(default_confirm_remove_message(Some(volid))),
            )
            .with_child(
                Row::new()
                    .with_flex_spacer()
                    .with_child(
                        Button::new(tr!("Cancel"))
                            .class("pwt-button-text")
                            .on_activate(link.callback(|_| Msg::Cancel)),
                    )
                    .with_child(
                        Button::new(tr!("Confirm"))
                            .class("pwt-button-text")
                            .on_activate(link.callback(|_| Msg::Remove)),
                    ),
            );

        let controller = link.page_controller().unwrap();

        controller.show_side_dialog(
            SideDialog::new()
                .location(pwt::touch::SideDialogLocation::Bottom)
                .with_child(content),
        );
    }
}

impl Component for PveVolumeActionDialog {
    type Message = Msg;
    type Properties = VolumeActionDialog;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Cancel => {
                log::info!("CLOSE");
                let controller = ctx.link().page_controller().unwrap();
                controller.close_side_dialog();
            }
            Msg::Remove => {
                log::info!("REMOVE");
                let controller = ctx.link().page_controller().unwrap();
                controller.close_side_dialog();
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        log::info!("ACVTION CHANGED");
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Column::new()
            .padding(2)
            .gap(2)
            .class(pwt::css::FlexFit)
            .with_child(
                Column::new()
                    .gap(1)
                    .with_child(
                        Container::new()
                            .class("pwt-font-size-title-large")
                            .with_child(tr!("Selection")),
                    )
                    .with_child(
                        Box::new(Row::new())
                            .class("pwt-font-size-title-small")
                            .with_child(Container::new().with_child(&props.item.volid))
                            .with_flex_spacer()
                            .with_child(
                                Container::new()
                                    .class("pwt-white-space-nowrap")
                                    .with_child(HumanByte::new_binary(props.item.size as f64)),
                            ),
                    ),
            )
            .with_child(
                Button::new(tr!("Restore"))
                    .icon_class("fa fa-undo")
                    .class("pwt-button-outline")
                    .disabled(true), // fixme
            )
            .with_child(
                ConfirmButton::remove_entry(props.item.volid.clone())
                    .icon_class("fa fa-trash-o")
                    .class("pwt-button-outline")
                    .on_activate(ctx.link().callback(|_| Msg::Remove)),
            )
            .with_child(
                Button::new(tr!("Show Configuration"))
                    .icon_class("fa fa-list-alt")
                    .class("pwt-button-outline"),
            )
            .into()
    }
}

impl From<VolumeActionDialog> for VNode {
    fn from(props: VolumeActionDialog) -> Self {
        let comp = VComp::new::<PveVolumeActionDialog>(Rc::new(props), None);
        VNode::from(comp)
    }
}
