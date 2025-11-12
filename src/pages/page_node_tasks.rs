use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew_router::hooks::use_navigator;

use pwt::prelude::*;
use pwt::widget::Column;

use proxmox_yew_comp::percent_encoding::percent_encode_component;

use crate::widgets::{TasksPanel, TopNavBar};

use pwt_macros::builder;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct PageNodeTasks {
    pub nodename: AttrValue,
}

impl PageNodeTasks {
    pub fn new(nodename: impl Into<AttrValue>) -> Self {
        Self {
            nodename: nodename.into(),
        }
    }
}

#[function_component]
pub fn PvePageNodeTasks(props: &PageNodeTasks) -> Html {
    let title = tr!("Node '{0}'", props.nodename);

    let navigator = use_navigator().unwrap();

    let base_url = format!("/nodes/{}/tasks", percent_encode_component(&props.nodename));

    Column::new()
        .class("pwt-fit")
        .with_child(
            TopNavBar::new()
                .title("Task List")
                .subtitle(title)
                .back(format!(
                    "/resources/node/{}",
                    percent_encode_component(&props.nodename)
                )),
        )
        .with_child(TasksPanel::new(base_url.clone()).on_show_task({
            let nodename = props.nodename.to_string();
            move |(upid, endtime): (String, Option<i64>)| {
                navigator.push(&crate::Route::NodeTaskStatus {
                    nodename: nodename.clone(),
                    upid,
                    endtime: endtime.unwrap_or(0),
                });
            }
        }))
        .into()
}

impl Into<VNode> for PageNodeTasks {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageNodeTasks>(Rc::new(self), None);
        VNode::from(comp)
    }
}
