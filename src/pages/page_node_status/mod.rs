use std::rc::Rc;

use anyhow::Error;
use serde::{Deserialize, Serialize};

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::props::StorageLocation;
use pwt::state::PersistentState;
use pwt::widget::{Column, TabBar, TabBarItem};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::http_get;

use pve_api_types::{ClusterNodeStatus, ClusterNodeStatusType};

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
    SetNodeStatus(Result<Vec<ClusterNodeStatus>, Error>),
}

pub struct PvePageNodeStatus {
    view_state: PersistentState<ViewState>,
    _cluster_status_guard: AsyncAbortGuard,
    cluster_node_status: Option<Vec<ClusterNodeStatus>>,
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

        let link = ctx.link().clone();

        Self {
            view_state,
            cluster_node_status: None,
            _cluster_status_guard: AsyncAbortGuard::spawn(async move {
                let result = http_get("/cluster/status", None).await;
                link.send_message(Msg::SetNodeStatus(result));
            }),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetViewState(view_state) => {
                self.view_state.update(view_state);
            }
            Msg::SetNodeStatus(result) => match result {
                Ok(status) => {
                    self.cluster_node_status = Some(status);
                }
                Err(err) => crate::show_failed_command_error(ctx.link(), err),
            },
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let node = props.node.clone();

        let mut standalone = true;
        if let Some(status) = &self.cluster_node_status {
            standalone = !status
                .iter()
                .any(|info| info.name == node && info.ty == ClusterNodeStatusType::Cluster)
        }

        let (active_tab, content): (_, Html) = match *self.view_state {
            ViewState::Dashboard => (
                "dashboard",
                NodeDashboardPanel::new(props.node.clone()).into(),
            ),
            ViewState::Services => (
                "services",
                NodeServicesPanel::new(props.node.clone(), standalone).into(),
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
