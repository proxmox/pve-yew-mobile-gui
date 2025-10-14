use std::rc::Rc;

use anyhow::Error;
use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, Container, Fa, List, ListTile};
use pwt::AsyncAbortGuard;

use pwt_macros::builder;

// fixme: implement filter
// fixme: implement reload on scroll down

use proxmox_yew_comp::http_get;
use proxmox_yew_comp::layout::render_loaded_data;
use proxmox_yew_comp::utils::{format_upid, render_epoch_short};

use pve_api_types::ListTasksResponse;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct TasksPanel {
    #[prop_or_default]
    pub base_url: AttrValue,

    #[builder_cb(IntoEventCallback, into_event_callback, (String, Option<i64>))]
    #[prop_or_default]
    /// Called when the task is opened
    pub on_show_task: Option<Callback<(String, Option<i64>)>>,
}

impl TasksPanel {
    pub fn new(base_url: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            base_url: base_url.into()
        })
    }
}

pub enum Msg {
    Load,
    LoadResult(Result<Vec<ListTasksResponse>, Error>),
}

pub struct PveTasksPanel {
    data: Option<Result<Rc<Vec<ListTasksResponse>>, String>>,
    load_guard: Option<AsyncAbortGuard>,
}

fn task_icon(task: &ListTasksResponse) -> Fa {
    if task.endtime.is_none() {
        return Fa::new("spinner").pulse();
    }

    match task.status.as_deref() {
        Some("OK") => Fa::new("info-circle").class(pwt::css::FontColor::Primary),
        Some(__) => Fa::new("exclamation-triangle").class(pwt::css::FontColor::Error),
        _ => Fa::new("question"),
    }
}

fn task_info(task: &ListTasksResponse) -> Html {
    Column::new()
        .gap(1)
        .with_child(
            Container::new()
                .class("pwt-font-size-title-medium")
                .with_child(format!("Task: {}", format_upid(&task.upid))),
        )
        .with_child(
            Container::new()
                .class("pwt-font-size-title-small")
                .with_child(render_epoch_short(task.endtime.unwrap_or(task.starttime))),
        )
        .into()
}

impl PveTasksPanel {
    fn view_task_list(&self, ctx: &Context<Self>, data: Rc<Vec<ListTasksResponse>>) -> Html {
        let props = ctx.props();
        let on_show_task = props.on_show_task.clone();

        List::new(data.len() as u64, move |pos| {
            let item = &data[pos as usize];
            ListTile::new()
                .interactive(true)
                .with_child(task_icon(item).margin_end(1).large_2x())
                .with_child(task_info(item))
                .with_child(Fa::new("chevron-right"))
                .onclick(on_show_task.clone().map(|cb| {
                    cb.reform({
                        let upid = item.upid.clone();
                        let endtime = item.endtime;
                        move |_| (upid.clone(), endtime)
                    })
                }))
        })
        .min_row_height(60)
        .separator(true)
        .class("pwt-flex-fit")
        .grid_template_columns("auto 1fr auto")
        .into()
    }
}

impl Component for PveTasksPanel {
    type Message = Msg;
    type Properties = TasksPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);

        Self {
            data: None,
            load_guard: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Load => {
                let link = ctx.link().clone();
                let url = props.base_url.clone();

                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&*url, None).await;
                    link.send_message(Msg::LoadResult(result));
                }));
            }
            Msg::LoadResult(result) => {
                self.data = Some(result.map(Rc::new).map_err(|err| err.to_string()));
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        render_loaded_data(&self.data, |data| {
            if data.is_empty() {
                Container::new()
                    .padding(2)
                    .with_child(tr!("List is empty."))
                    .into()
            } else {
                self.view_task_list(ctx, Rc::clone(data))
            }
        })
    }
}

impl From<TasksPanel> for VNode {
    fn from(props: TasksPanel) -> Self {
        let comp = VComp::new::<PveTasksPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
