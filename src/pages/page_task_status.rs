use std::rc::Rc;

use anyhow::{format_err, Error};

use gloo_timers::callback::Timeout;
use proxmox_yew_comp::LogView;
use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::widget::{Column, List, ListTile, TabBar, TabBarItem};
use pwt::{prelude::*, AsyncAbortGuard};

use pve_api_types::{IsRunning, TaskStatus};

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

#[derive(Copy, Clone, PartialEq)]
pub enum ViewState {
    Output,
    Status,
}

pub struct PvePageTaskStatus {
    active: bool,
    view_state: ViewState,
    status: Result<TaskStatus, Error>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
}

pub enum Msg {
    SetViewState(ViewState),
    Load,
    LoadResult(Result<TaskStatus, Error>),
}

impl PvePageTaskStatus {
    fn view_output(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let active = self.active;

        let url = format!(
            "{}/{}/log",
            props.base_url,
            percent_encode_component(&props.task_id),
        );

        LogView::new(url)
            .padding(2)
            .class("pwt-flex-fill")
            .active(active)
            .into()
    }

    fn view_status(&self, ctx: &Context<Self>) -> Html {
        match &self.status {
            Ok(status) => {
                let tiles: Vec<ListTile> = Vec::new();

                List::new(tiles.len() as u64, move |pos| {
                    tiles[pos as usize].clone().into()
                })
                .into()
            }
            Err(err) => pwt::widget::error_message(&err.to_string()).into(),
        }
    }
}

impl Component for PvePageTaskStatus {
    type Message = Msg;
    type Properties = PageTaskStatus;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let active = props.endtime.map(|endtime| endtime == 0).unwrap_or(true);

        ctx.link().send_message(Msg::Load);

        Self {
            active,
            view_state: ViewState::Output,
            status: Err(format_err!("no data loaded")),
            load_guard: None,
            reload_timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::SetViewState(view_state) => {
                self.view_state = view_state;
                true
            }
            Msg::Load => {
                let link = ctx.link().clone();
                let url = format!(
                    "{}/{}/status",
                    props.base_url,
                    percent_encode_component(&props.task_id),
                );
                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = proxmox_yew_comp::http_get(url, None).await;
                    link.send_message(Msg::LoadResult(result));
                }));
                false
            }
            Msg::LoadResult(result) => {
                let link = ctx.link().clone();
                self.active = match &result {
                    Ok(status) => status.status == IsRunning::Running,
                    Err(_) => true,
                };
                self.status = result;
                if self.active {
                    self.reload_timeout = Some(Timeout::new(1_000, move || {
                        link.send_message(Msg::Load);
                    }));
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let tabbar = TabBar::new()
            .class(pwt::css::JustifyContent::Center)
            .with_item(
                TabBarItem::new().label("Output").key("output").on_activate(
                    ctx.link()
                        .callback(|_| Msg::SetViewState(ViewState::Output)),
                ),
            )
            .with_item(
                TabBarItem::new().label("Status").key("status").on_activate(
                    ctx.link()
                        .callback(|_| Msg::SetViewState(ViewState::Status)),
                ),
            );

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title("Task Status")
                    //.subtitle(&props.title)
                    .back(&props.back),
            )
            .with_child(tabbar)
            .with_child(match self.view_state {
                ViewState::Output => self.view_output(ctx),
                ViewState::Status => self.view_status(ctx),
            })
            .into()
    }
}

impl Into<VNode> for PageTaskStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageTaskStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}
