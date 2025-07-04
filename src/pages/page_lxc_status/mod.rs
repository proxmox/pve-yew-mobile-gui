use std::rc::Rc;

use pwt::props::StorageLocation;
use serde::{Deserialize, Serialize};

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::state::PersistentState;
use pwt::widget::{Column, TabBar, TabBarItem};

use crate::widgets::{GuestBackupPanel, TopNavBar};

mod dashboard_panel;
pub use dashboard_panel::LxcDashboardPanel;

mod resources_panel;
pub use resources_panel::LxcResourcesPanel;

#[derive(Clone, PartialEq, Properties)]
pub struct PageLxcStatus {
    vmid: u32,
    node: AttrValue,
}

impl PageLxcStatus {
    pub fn new(node: impl Into<AttrValue>, vmid: u32) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum ViewState {
    #[default]
    Dashboard,
    Options,
    Backup,
}

pub enum Msg {
    SetViewState(ViewState),
}

pub struct PvePageLxcStatus {
    view_state: PersistentState<ViewState>,
}

impl Component for PvePageLxcStatus {
    type Message = Msg;
    type Properties = PageLxcStatus;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let view_state = PersistentState::new(StorageLocation::session(format!(
            "ct-{}-status-tab-bar-state",
            props.vmid
        )));

        Self { view_state }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetViewState(view_state) => {
                self.view_state.update(view_state);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (active_tab, content): (_, Html) = match *self.view_state {
            ViewState::Dashboard => (
                "dashboard",
                LxcDashboardPanel::new(props.node.clone(), props.vmid).into(),
            ),
            ViewState::Backup => (
                "backup",
                GuestBackupPanel::new(props.node.clone(), props.vmid).into(),
            ),
            ViewState::Options => (
                "options",
                //LxcConfigPanel::new(props.node.clone(), props.vmid).into(),
                html! {"OPTIONS"},
            ),
        };

        let tab_bar = TabBar::new()
            .class(pwt::css::JustifyContent::Center)
            .active(active_tab)
            .with_item(
                TabBarItem::new()
                    .label("Dashboard")
                    .key("dashboard")
                    .on_activate(
                        ctx.link()
                            .callback(|_| Msg::SetViewState(ViewState::Dashboard)),
                    ),
            )
            .with_item(
                TabBarItem::new()
                    .label("Options")
                    .key("options")
                    .on_activate(
                        ctx.link()
                            .callback(|_| Msg::SetViewState(ViewState::Options)),
                    ),
            )
            .with_item(
                TabBarItem::new().label("Backup").key("backup").on_activate(
                    ctx.link()
                        .callback(|_| Msg::SetViewState(ViewState::Backup)),
                ),
            );

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title(format!("CT {}", props.vmid))
                    .back(true),
            )
            .with_child(tab_bar)
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageLxcStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageLxcStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}
