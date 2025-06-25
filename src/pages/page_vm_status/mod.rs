mod vm_hardware_panel;
pub use vm_hardware_panel::VmHardwarePanel;

mod vm_config_panel;
pub use vm_config_panel::VmConfigPanel;

mod dashboard_panel;
pub use dashboard_panel::VmDashboardPanel;

use std::rc::Rc;

use serde::{Deserialize, Serialize};

use pwt::props::StorageLocation;
use pwt::state::PersistentState;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, Container, TabBar, TabBarItem};

use crate::widgets::TopNavBar;

#[derive(Clone, PartialEq, Properties)]
pub struct PageVmStatus {
    vmid: u32,
    node: AttrValue,
}

impl PageVmStatus {
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
pub struct PvePageVmStatus {
    view_state: PersistentState<ViewState>,
}

pub enum Msg {
    SetViewState(ViewState),
}

impl PvePageVmStatus {
    fn view_backup(&self, _ctx: &Context<Self>) -> Html {
        Container::new().with_child("Backup").into()
    }
}

impl Component for PvePageVmStatus {
    type Message = Msg;
    type Properties = PageVmStatus;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let view_state = PersistentState::new(StorageLocation::session(format!(
            "vm-{}-status-tab-bar-state",
            props.vmid
        )));

        Self { view_state }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::SetViewState(view_state) => {
                self.view_state.update(view_state);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (active_tab, content) = match *self.view_state {
            ViewState::Dashboard => (
                "dashboard",
                VmDashboardPanel::new(props.node.clone(), props.vmid).into(),
            ),
            ViewState::Backup => ("backup", self.view_backup(ctx)),
            ViewState::Options => (
                "options",
                VmConfigPanel::new(props.node.clone(), props.vmid).into(),
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
                    .title(format!("VM {}", props.vmid))
                    .back(true),
            )
            .with_child(tab_bar)
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageVmStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageVmStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}
