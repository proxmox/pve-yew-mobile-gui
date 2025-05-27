use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, Container};

use crate::widgets::TopNavBar;

#[derive(Clone, PartialEq, Properties)]
pub struct PageContainerStatus {
    vmid: u64,
}

impl PageContainerStatus {
    pub fn new(vmid: u64) -> Self {
        Self { vmid }
    }
}

pub struct PvePageContainerStatus {}

pub enum Msg {}

impl Component for PvePageContainerStatus {
    type Message = Msg;
    type Properties = PageContainerStatus;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = Column::new()
            .padding(2)
            .with_child(format!("This is the status for container {}", props.vmid));

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title(format!("CT {}", props.vmid))
                    .back("/resources"),
            )
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageContainerStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageContainerStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}
