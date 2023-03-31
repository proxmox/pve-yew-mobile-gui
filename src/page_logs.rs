use std::rc::Rc;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::form::{Checkbox, Form, FormContext};
use pwt::widget::Column;

use crate::TopNavBar;

fn get_form_context() -> FormContext {
    static mut FORM_CTX: Option<FormContext> = None;

    unsafe {
        if FORM_CTX.is_none() {
            FORM_CTX = Some(FormContext::new());
        }
        FORM_CTX.as_ref().unwrap().clone()
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct PageLogs {}

impl PageLogs {
    pub fn new() -> Self {
        Self {}
    }
}
pub struct PvePageLogs {
    form_ctx: FormContext,
}

pub enum Msg {
    Reload,
}

impl PvePageLogs {
    fn view_task_log(&self, _ctx: &Context<Self>) -> Html {
        html!("TASK LOG")
    }

    fn view_cluster_log(&self, _ctx: &Context<Self>) -> Html {
        html!("Cluster LOG")
    }
}

impl Component for PvePageLogs {
    type Message = Msg;
    type Properties = PageLogs;

    fn create(ctx: &Context<Self>) -> Self {
        let form_ctx = get_form_context().on_change(ctx.link().callback(|_| Msg::Reload));

        Self { form_ctx }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Reload => true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let task_log = self
            .form_ctx
            .read()
            .get_field_value("what")
            .map(|value| value.as_str().unwrap_or("task") == "task")
            .unwrap_or(true);

        let switch = Form::new()
            .class("pwt-d-flex pwt-justify-content-space-evenly pwt-p-2 pwt-navbar")
            .form_context(get_form_context())
            .with_child(
                Column::new()
                    .class("pwt-align-items-center")
                    .with_child("Task Log")
                    .with_child(Checkbox::radio().default(true).name("what").value("task")),
            )
            .with_child(
                Column::new()
                    .class("pwt-align-items-center")
                    .with_child("Cluster Log")
                    .with_child(Checkbox::radio().name("what").value("cluster")),
            );

        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new().title("Logs"))
            .with_child(switch)
            .with_child(if task_log {
                self.view_task_log(ctx)
            } else {
                self.view_cluster_log(ctx)
            })
            .into()
    }
}

impl Into<VNode> for PageLogs {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageLogs>(Rc::new(self), None);
        VNode::from(comp)
    }
}
