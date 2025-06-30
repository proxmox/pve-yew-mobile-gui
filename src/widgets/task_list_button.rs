use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;

use proxmox_yew_comp::common_api_types::ProxmoxUpid;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{VComp, VNode};

use pwt::widget::{Button, Card, Column, Container};
use pwt::{prelude::*, AsyncAbortGuard};

use proxmox_yew_comp::http_get;
use proxmox_yew_comp::percent_encoding::percent_encode_component;
use proxmox_yew_comp::utils::format_upid;

use pve_api_types::{IsRunning, TaskStatus};

use pwt_macros::builder;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct TasksListButton {
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub running_upid: Option<AttrValue>,

    #[builder_cb(IntoEventCallback, into_event_callback, (String, Option<i64>))]
    #[prop_or_default]
    /// Called to show the running_upid
    pub on_show_task: Option<Callback<(String, Option<i64>)>>,

    #[builder_cb(IntoEventCallback, into_event_callback, MouseEvent)]
    #[prop_or_default]
    /// Called to show the task list
    pub on_show_task_list: Option<Callback<MouseEvent>>,
}

impl TasksListButton {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}
pub struct ProxmoxTaskListButton {
    check_task_status_timeout: Option<Timeout>,
    task_status_guard: Option<AsyncAbortGuard>,
    running_upid: Option<AttrValue>,
    last_task_status: Option<String>,
}

pub enum Msg {
    CheckTaskStatus,
    TaskStatusResult(AttrValue, Result<TaskStatus, Error>),
}

impl ProxmoxTaskListButton {
    fn update_upid(&mut self, ctx: &Context<Self>, upid: &str) {
        self.running_upid = Some(upid.to_string().into());
        let task_descr = format_upid(&upid);
        self.last_task_status = Some(format!("{task_descr} (running)"));
        self.check_running_task_status(ctx);
    }

    fn check_running_task_status(&mut self, ctx: &Context<Self>) {
        let props = ctx.props();
        let upid = match &self.running_upid {
            Some(upid) => upid.clone(),
            None => return,
        };

        let node = match upid.parse::<ProxmoxUpid>() {
            Ok(upid) => upid.node,
            Err(err) => {
                // fixme: log error
                return;
            }
        };

        let url = format!(
            "/nodes/{}/tasks/{}/status",
            percent_encode_component(&node),
            percent_encode_component(&upid)
        );
        let link = ctx.link().clone();
        self.task_status_guard = Some(AsyncAbortGuard::spawn(async move {
            let result = http_get(&url, None).await;
            link.send_message(Msg::TaskStatusResult(upid.clone(), result));
        }));
    }
}

impl Component for ProxmoxTaskListButton {
    type Message = Msg;
    type Properties = TasksListButton;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let mut me = Self {
            check_task_status_timeout: None,
            task_status_guard: None,
            running_upid: None,
            last_task_status: None,
        };
        if let Some(upid) = &props.running_upid {
            me.update_upid(ctx, upid);
        }
        me
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.running_upid != old_props.running_upid {
            if let Some(upid) = &props.running_upid {
                self.update_upid(ctx, upid);
            } else {
                self.last_task_status = None;
            }
        }
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::CheckTaskStatus => {
                self.check_running_task_status(ctx);
            }
            Msg::TaskStatusResult(upid, result) => {
                if Some(&upid) != self.running_upid.as_ref() {
                    return false;
                }
                let running = match &result {
                    Ok(status) if status.status == IsRunning::Running => true,
                    _ => false,
                };

                if running {
                    let link = ctx.link().clone();
                    self.check_task_status_timeout = Some(Timeout::new(1000, move || {
                        link.send_message(Msg::CheckTaskStatus);
                    }));
                } else {
                    let task_descr = format_upid(&upid);
                    let exit_status = match result {
                        Ok(status) => status.exitstatus,
                        Err(_) => None,
                    };
                    self.last_task_status = Some(format!(
                        "{}: {task_descr} ({})",
                        tr!("Finished"),
                        exit_status.as_deref().unwrap_or("unknown")
                    ));
                    self.running_upid = None;
                    self.check_task_status_timeout = None;
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = match &self.last_task_status {
            Some(last_task_status) => Some(
                Container::new()
                    .class("pwt-font-size-title-medium")
                    .with_child(last_task_status),
            ),
            None => None,
        };

        Column::new()
            .gap(2)
            .with_child(Button::new(tr!("Task List")).on_activate(props.on_show_task_list.clone()))
            .with_optional_child(content)
            .into()
    }
}

impl From<TasksListButton> for VNode {
    fn from(props: TasksListButton) -> Self {
        let comp = VComp::new::<ProxmoxTaskListButton>(Rc::new(props), None);
        VNode::from(comp)
    }
}
