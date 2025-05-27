use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::Column;

use crate::TopNavBar;

use pwt_macros::builder;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct PageTasks {
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub title: Option<AttrValue>,

    pub url: Option<AttrValue>,

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub back: Option<AttrValue>,
}

impl PageTasks {
    pub fn new(url: impl Into<AttrValue>) -> Self {
        yew::props!(Self { url: url.into() })
    }
}

pub struct PvePageTasks {}

pub enum Msg {}

impl Component for PvePageTasks {
    type Message = Msg;
    type Properties = PageTasks;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title("Task List")
                    .subtitle(&props.title)
                    .back(&props.back),
            )
            .with_child("TASK LIST")
            .into()
    }
}

impl Into<VNode> for PageTasks {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageTasks>(Rc::new(self), None);
        VNode::from(comp)
    }
}
