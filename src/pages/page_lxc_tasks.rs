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
pub struct PageLxcTasks {
    pub vmid: u32,
    pub nodename: AttrValue,
}

impl PageLxcTasks {
    pub fn new(nodename: impl Into<AttrValue>, vmid: u32) -> Self {
        Self {
            nodename: nodename.into(),
            vmid,
        }
    }
}

#[function_component]
pub fn PvePageLxcTasks(props: &PageLxcTasks) -> Html {
    let title = format!("CT {}", props.vmid);

    let navigator = use_navigator().unwrap();

    let base_url = format!(
        "/nodes/{}/tasks?vmid={}",
        percent_encode_component(&props.nodename),
        props.vmid,
    );

    Column::new()
        .class("pwt-fit")
        .with_child(
            TopNavBar::new()
                .title("Task List")
                .subtitle(title)
                .back(true),
        )
        .with_child(TasksPanel::new(base_url.clone()).on_show_task({
            let vmid = props.vmid;
            let nodename = props.nodename.to_string();
            move |(upid, endtime): (String, Option<i64>)| {
                navigator.push(&crate::Route::LxcTaskStatus {
                    vmid,
                    nodename: nodename.clone(),
                    upid,
                    endtime: endtime.unwrap_or(0),
                });
            }
        }))
        .into()
}

impl Into<VNode> for PageLxcTasks {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageLxcTasks>(Rc::new(self), None);
        VNode::from(comp)
    }
}
