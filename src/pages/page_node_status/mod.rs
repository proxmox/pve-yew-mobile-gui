use std::rc::Rc;

use pwt::props::StorageLocation;
use pwt::state::PersistentState;
use serde::{Deserialize, Serialize};

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, TabBar, TabBarItem};

use crate::widgets::TopNavBar;

mod dashboard_panel;
pub use dashboard_panel::NodeDashboardPanel;

mod services_panel;
pub use services_panel::NodeServicesPanel;

mod updates_panel;
pub use updates_panel::NodeUpdatesPanel;

#[derive(Clone, PartialEq, Properties)]
pub struct PageNodeStatus {
    node: AttrValue,
}

impl PageNodeStatus {
    pub fn new(nodename: impl Into<AttrValue>) -> Self {
        Self {
            node: nodename.into(),
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum ViewState {
    #[default]
    Dashboard,
    Services,
    Updates,
}

pub enum Msg {
    SetViewState(ViewState),
}

pub struct PvePageNodeStatus {
    view_state: PersistentState<ViewState>,
}

impl Component for PvePageNodeStatus {
    type Message = Msg;
    type Properties = PageNodeStatus;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let view_state = PersistentState::new(StorageLocation::session(format!(
            "node-{}-status-tab-bar-state",
            props.node
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
                NodeDashboardPanel::new(props.node.clone()).into(),
            ),
            ViewState::Services => (
                "services",
                NodeServicesPanel::new(props.node.clone()).into(),
            ),
            ViewState::Updates => ("updates", NodeUpdatesPanel::new(props.node.clone()).into()),
        };

        let tab_bar = TabBar::new()
            .class(pwt::css::JustifyContent::Center)
            .active(active_tab)
            .with_item(
                TabBarItem::new()
                    .label(tr!("Dashboard"))
                    .key("dashboard")
                    .on_activate(
                        ctx.link()
                            .callback(|_| Msg::SetViewState(ViewState::Dashboard)),
                    ),
            )
            .with_item(
                TabBarItem::new()
                    .label(tr!("Services"))
                    .key("services")
                    .on_activate(
                        ctx.link()
                            .callback(|_| Msg::SetViewState(ViewState::Services)),
                    ),
            )
            .with_item(
                TabBarItem::new()
                    .label(tr!("Updates"))
                    .key("updates")
                    .on_activate(
                        ctx.link()
                            .callback(|_| Msg::SetViewState(ViewState::Updates)),
                    ),
            );

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title(format!("Node {}", props.node))
                    .back(true),
            )
            .with_child(tab_bar)
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageNodeStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageNodeStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}
