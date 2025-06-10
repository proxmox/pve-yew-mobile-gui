use std::rc::Rc;

use anyhow::Error;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, Container, Fa, List, ListTile};
use pwt::AsyncAbortGuard;

// fixme: implement filter
// fixme: implement reload on scroll down

use proxmox_yew_comp::http_get;
use proxmox_yew_comp::utils::render_epoch_short;

use pve_api_types::ListTasksResponse;

#[derive(Clone, PartialEq, Properties)]
pub struct TasksPanel {
    #[prop_or_default]
    pub base_url: AttrValue,
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
    data: Rc<Vec<ListTasksResponse>>,
    load_guard: Option<AsyncAbortGuard>,
}

fn task_icon(task: &ListTasksResponse) -> Fa {
    if task.endtime.is_none() {
        return Fa::new("spinner").pulse();
    }

    if let Some(text) = task.status.as_deref() {
        if text == "OK" {
            return Fa::new("info-circle").class(pwt::css::FontColor::Primary);
        }
        return Fa::new("exclamation-triangle").class(pwt::css::FontColor::Error);
    }

    Fa::new("question")
}

fn task_info(task: &ListTasksResponse) -> Html {
    let status = if let Some(endtime) = task.endtime {
        render_epoch_short(endtime)
    } else {
        render_epoch_short(task.starttime)
    };

    Column::new()
        .gap(1)
        .with_child(
            Container::new()
                .class("pwt-font-size-title-medium")
                .with_child(format!("Task: {}", task.ty)),
        )
        .with_child(
            Container::new()
                .class("pwt-font-size-title-small")
                .with_child(status),
        )
        .into()
}

impl Component for PveTasksPanel {
    type Message = Msg;
    type Properties = TasksPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);

        Self {
            data: Rc::new(Vec::new()),
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
                match result {
                    Ok(list) => self.data = Rc::new(list),
                    Err(err) => {
                        // fixme:
                        log::error!("load error {err}");
                    }
                }
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if self.data.is_empty() {
            return Container::new()
                .padding(2)
                .with_child("list contains no data")
                .into();
        }

        let data = Rc::clone(&self.data);
        List::new(self.data.len() as u64, move |pos| {
            let item = &data[pos as usize];
            ListTile::new()
                .interactive(true)
                .with_child(task_icon(item).margin_end(1).large_2x())
                .with_child(task_info(item))
                .with_child(Fa::new("chevron-down"))
        })
        .min_row_height(60)
        .separator(true)
        .class("pwt-flex-fit")
        .grid_template_columns("auto 1fr auto")
        .into()
    }
}

impl From<TasksPanel> for VNode {
    fn from(props: TasksPanel) -> Self {
        let comp = VComp::new::<PveTasksPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
