use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::Column;

use crate::widgets::{TasksPanel, TopNavBar};

use pwt_macros::builder;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct PageTasks {
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub title: Option<AttrValue>,

    pub base_url: AttrValue,

    #[builder_cb(IntoEventCallback, into_event_callback, (String, Option<i64>))]
    #[prop_or_default]
    /// Called when the task is opened
    pub on_show_task: Option<Callback<(String, Option<i64>)>>,
}

impl PageTasks {
    pub fn new(base_url: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            base_url: base_url.into()
        })
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
                    .back(true),
            )
            .with_child(
                TasksPanel::new(props.base_url.clone()).on_show_task(props.on_show_task.clone()),
            )
            .into()
    }
}

impl Into<VNode> for PageTasks {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageTasks>(Rc::new(self), None);
        VNode::from(comp)
    }
}
