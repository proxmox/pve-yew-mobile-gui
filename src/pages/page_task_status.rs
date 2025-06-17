use std::rc::Rc;

use proxmox_yew_comp::LogView;
use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::Column;

use proxmox_yew_comp::percent_encoding::percent_encode_component;

use crate::widgets::TopNavBar;

use pwt_macros::builder;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct PageTaskStatus {
    pub base_url: AttrValue,

    pub task_id: String,

    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub endtime: Option<i64>,

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub back: Option<AttrValue>,
}

impl PageTaskStatus {
    pub fn new(base_url: impl Into<AttrValue>, task_id: impl Into<String>) -> Self {
        yew::props!(Self {
            base_url: base_url.into(),
            task_id: task_id.into(),
        })
    }
}

pub struct PvePageTaskStatus {
    active: bool,
}

pub enum Msg {}

impl Component for PvePageTaskStatus {
    type Message = Msg;
    type Properties = PageTaskStatus;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let active = props.endtime.map(|endtime| endtime == 0).unwrap_or(true);
        Self { active }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let active = self.active;

        let url = format!(
            "{}/{}/log",
            props.base_url,
            percent_encode_component(&props.task_id),
        );

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title("Task Status")
                    //.subtitle(&props.title)
                    .back(&props.back),
            )
            .with_child(
                LogView::new(url)
                    .padding(2)
                    .class("pwt-flex-fill")
                    .active(active),
            )
            //.with_child(TaskViewer::new(props.task_id.clone()).base_url(props.base_url.clone()))
            .into()
    }
}

impl Into<VNode> for PageTaskStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageTaskStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}
