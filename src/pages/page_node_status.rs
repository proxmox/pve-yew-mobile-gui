use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Card, Column};

use crate::widgets::TopNavBar;

use proxmox_yew_comp::percent_encoding::percent_encode_component;

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

impl PvePageNodeStatus {
    fn task_button(&self, ctx: &Context<Self>) -> Html {
        Card::new()
            .padding(2)
            .class("pwt-d-flex")
            .class("pwt-interactive")
            .class(pwt::css::JustifyContent::Center)
            .with_child("Task List")
            .onclick({
                let props = ctx.props();
                let url = format!(
                    "/resources/node/{}/tasks",
                    percent_encode_component(&props.nodename),
                );
                move |_| crate::goto_location(&url)
            })
            .into()
    }
}

impl Component for PvePageNodeStatus {
    type Message = Msg;
    type Properties = PageNodeStatus;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = Column::new()
            .class(pwt::css::FlexFit)
            .padding(2)
            //.with_child(format!("This is the status for node {}", props.nodename))
            .with_child(self.task_button(ctx));

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title(format!("Node {}", props.nodename))
                    .back("/resources/node"),
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
