use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, Container};

use crate::TopNavBar;

#[derive(Clone, PartialEq, Properties)]
pub struct PageNodeStatus {
    nodename: AttrValue,
}

impl PageNodeStatus {
    pub fn new(nodename: impl Into<AttrValue>) -> Self {
        Self {
            nodename: nodename.into(),
        }
    }
}

pub struct PvePageNodeStatus {}

pub enum Msg {}

impl Component for PvePageNodeStatus {
    type Message = Msg;
    type Properties = PageNodeStatus;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = Column::new()
            .padding(2)
            .with_child(format!("This is the status for node {}", props.nodename));

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title(format!("Node {}", props.nodename))
                    .back("/resources"),
            )
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageNodeStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageNodeStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}
