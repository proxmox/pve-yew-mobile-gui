use std::rc::Rc;

use anyhow::{format_err, Error};

use gloo_timers::callback::Timeout;
use proxmox_yew_comp::LogView;
use pwt::widget::menu::MenuItem;
use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::widget::{Column, Container, List, ListTile, TabBar, TabBarItem};
use pwt::{prelude::*, AsyncAbortGuard};

use pve_api_types::{IsRunning, TaskStatus};

use proxmox_yew_comp::percent_encoding::percent_encode_component;
use proxmox_yew_comp::utils::format_upid;

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
    stop_task_guard: Option<AsyncAbortGuard>,
}

pub enum Msg {
    SetViewState(ViewState),
    Load,
    LoadResult(Result<TaskStatus, Error>),
    StopTask,
}

pub fn status_list_tile(title: impl Into<AttrValue>, subtitle: impl Into<AttrValue>) -> ListTile {
    ListTile::new()
        .class(pwt::css::AlignItems::Center)
        .class("pwt-gap-1")
        .border_bottom(true)
        .with_child({
            let mut column = Column::new().gap(1);

            if let Some(title) = title.into_prop_value() {
                column.add_child(
                    Container::new()
                        .class("pwt-font-size-title-medium")
                        .key("title")
                        .with_child(title.into()),
                );
            }

            if let Some(subtitle) = subtitle.into_prop_value() {
                column.add_child(
                    Container::new()
                        .class("pwt-font-size-title-small")
                        .key("subtitle")
                        .with_child(subtitle.into()),
                );
            }
            column
        })
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
        let props = ctx.props();
        match &self.status {
            Ok(status) => {
                let mut tiles: Vec<ListTile> = Vec::new();

                tiles.push(status_list_tile(
                    tr!("Status"),
                    match (&status.status, &status.exitstatus) {
                        (IsRunning::Running, _) => "running".into(),
                        (IsRunning::Stopped, Some(msg)) => format!("{} ({})", tr!("stopped"), msg),
                        (IsRunning::Stopped, None) => {
                            format!("{} ({})", tr!("stopped"), tr!("unknown"))
                        }
                    },
                ));

                tiles.push(status_list_tile(tr!("Task type"), status.ty.clone()));

                tiles.push(status_list_tile(tr!("Worker ID"), status.id.clone()));

                tiles.push(status_list_tile(tr!("User name"), status.user.clone()));

                tiles.push(status_list_tile(tr!("Node"), status.node.clone()));

                tiles.push(status_list_tile(
                    tr!("Start Time"),
                    proxmox_yew_comp::utils::render_epoch_short(status.starttime),
                ));

                if let Some(endtime) = props.endtime {
                    tiles.push(status_list_tile(
                        tr!("Duration"),
                        proxmox_yew_comp::utils::format_duration_human(
                            (endtime - status.starttime) as f64,
                        ),
                    ));
                }

                tiles.push(status_list_tile(tr!("Unique task ID"), status.upid.clone()));

                List::new(tiles.len() as u64, move |pos| {
                    tiles[pos as usize].clone().into()
                })
                .class(pwt::css::FlexFit)
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
            stop_task_guard: None,
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
            Msg::StopTask => {
                let link = ctx.link().clone();
                let url = format!(
                    "{}/{}",
                    props.base_url,
                    percent_encode_component(&props.task_id),
                );
                self.stop_task_guard = Some(AsyncAbortGuard::spawn(async move {
                    let _ = proxmox_yew_comp::http_delete(url, None).await; // ignore errors
                    link.send_message(Msg::Load);
                }));
                false
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
                    .subtitle(format_upid(&props.task_id))
                    .back(true)
                    .with_item(
                        MenuItem::new(tr!("Stop"))
                            .icon_class("fa fa-stop")
                            .disabled(!self.active)
                            .on_select(ctx.link().callback(|_| Msg::StopTask)),
                    ),
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
