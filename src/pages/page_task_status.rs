use std::rc::Rc;

use proxmox_yew_comp::LogView;
use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, List, ListTile, TabBar, TabBarItem};

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
}

pub enum Msg {
    SetViewState(ViewState),
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
        let tiles: Vec<ListTile> = Vec::new();

        List::new(tiles.len() as u64, move |pos| {
            tiles[pos as usize].clone().into()
        })
        .into()
    }
}

impl Component for PvePageTaskStatus {
    type Message = Msg;
    type Properties = PageTaskStatus;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let active = props.endtime.map(|endtime| endtime == 0).unwrap_or(true);
        Self {
            active,
            view_state: ViewState::Output,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetViewState(view_state) => {
                self.view_state = view_state;
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
