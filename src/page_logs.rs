use std::rc::Rc;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{error_message, Column};

use crate::TopNavBar;

#[derive(Clone, PartialEq, Properties)]
pub struct PageLogs {}

impl PageLogs {
    pub fn new() -> Self {
        Self {}
    }
}
pub struct PvePageLogs {

}

impl Component for PvePageLogs {
    type Message = ();
    type Properties = PageLogs;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new().title("Logs"))
            .with_child("LOGS")
            .into()
    }
}

impl Into<VNode> for PageLogs {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageLogs>(Rc::new(self), None);
        VNode::from(comp)
    }
}
