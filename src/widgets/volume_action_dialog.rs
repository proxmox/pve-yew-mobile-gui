use std::rc::Rc;

use proxmox_human_byte::HumanByte;
use yew::html::IntoEventCallback;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::MaterialAppScopeExt;
use pwt::widget::{Button, Column, Container, Row};

use pwt_macros::builder;

use proxmox_yew_comp::ConfirmButton;

use crate::StorageEntry;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct VolumeActionDialog {
    item: StorageEntry,

    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    /// Called when the task is opened
    pub on_remove: Option<Callback<()>>,

    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    /// Called when the task is opened
    pub on_show_config: Option<Callback<()>>,
}

impl VolumeActionDialog {
    pub fn new(item: StorageEntry) -> Self {
        yew::props!(Self { item })
    }
}

pub struct PveVolumeActionDialog {}

impl Component for PveVolumeActionDialog {
    type Message = ();
    type Properties = VolumeActionDialog;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let controller = ctx.link().page_controller().unwrap();

        let wrap_callback = |cb: Option<Callback<_>>| {
            let controller = controller.clone();
            Callback::from(move |()| {
                controller.close_side_dialog();
                if let Some(cb) = &cb {
                    cb.emit(())
                };
            })
        };

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
            /* fixme: implement restore?
            .with_child(
                Button::new(tr!("Restore"))
                    .icon_class("fa fa-undo")
                    .class("pwt-button-outline")
                    .disabled(true), // fixme
            )
            */
            .with_child(
                ConfirmButton::remove_entry(props.item.volid.clone())
                    .icon_class("fa fa-trash-o")
                    .class("pwt-button-outline")
                    .on_activate(wrap_callback(props.on_remove.clone())),
            )
            .with_child(
                Button::new(tr!("Show Configuration"))
                    .icon_class("fa fa-list-alt")
                    .class("pwt-button-outline")
                    .on_activate({
                        let cb = wrap_callback(props.on_show_config.clone());
                        move |_| cb.emit(())
                    }),
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
