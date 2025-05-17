use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::Fab;
use pwt::widget::{Column, Container};

use crate::TopNavBar;

#[derive(Clone, PartialEq, Properties)]
pub struct PageVmStatus {
    vmid: u64,
}

impl PageVmStatus {
    pub fn new(vmid: u64) -> Self {
        Self { vmid }
    }
}

pub struct PvePageVmStatus {}

pub enum Msg {}

impl Component for PvePageVmStatus {
    type Message = Msg;
    type Properties = PageVmStatus;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = Column::new()
            .padding(2)
            .with_child(format!("This is the VmStatus for VM {}", props.vmid));

        let fab = Container::new()
            .class("pwt-position-absolute")
            .style("right", "var(--pwt-spacer-2)")
            .style("bottom", "var(--pwt-spacer-2)")
            .with_child(
                Fab::new("fa fa-calendar").class("pwt-scheme-primary"), //.on_click(ctx.link().callback(|_| Msg::ShowDialog))
            );

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title(format!("VM {}", props.vmid))
                    .back("/resources"),
            )
            .with_child(content)
            .with_child(fab)
            .into()
    }
}

impl Into<VNode> for PageVmStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageVmStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}
