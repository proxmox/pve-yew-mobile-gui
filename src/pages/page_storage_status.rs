use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::Column;

use crate::widgets::TopNavBar;

#[derive(Clone, PartialEq, Properties)]
pub struct PageStorageStatus {
    name: AttrValue,
}

impl PageStorageStatus {
    pub fn new(name: impl Into<AttrValue>) -> Self {
        Self { name: name.into() }
    }
}

pub struct PvePageStorageStatus {}

pub enum Msg {}

impl Component for PvePageStorageStatus {
    type Message = Msg;
    type Properties = PageStorageStatus;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = Column::new()
            .padding(2)
            .with_child(format!("This is the status for storage {}", props.name));

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title(format!("Storage {}", props.name))
                    .back(true),
            )
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageStorageStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageStorageStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}
