use std::rc::Rc;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, ListTile};

use crate::widgets::TopNavBar;

#[derive(Clone, PartialEq, Properties)]
pub struct PageConfiguration {}

impl PageConfiguration {
    pub fn new() -> Self {
        Self {}
    }
}
pub struct PvePageConfiguration {}

static CONFIGS: &[(&'static str, &'static str, fn() -> Html)] = &[
    ("fa fa-fw fa-server", "Cluster", || {
        html! {}
    }),
    ("fa fa-fw fa-gear", "Options", || {
        html! {}
    }),
    ("fa fa-fw fa-database", "Storage", || {
        html! {}
    }),
    ("fa fa-fw fa-floppy-o", "Backup", || {
        html! {}
    }),
    ("fa fa-fw fa-retweet", "Replication", || {
        html! {}
    }),
    ("fa fa-fw fa-unlock", "Permissions", || {
        html! {}
    }),
    ("fa fa-fw fa-heartbeat", "High Availability", || {
        html! {}
    }),
    ("fa fa-fw fa-certificate", "ACME", || {
        html! {}
    }),
    ("fa fa-fw fa-shield", "Firewall", || {
        html! {}
    }),
    ("fa fa-fw fa-bar-chart", "Metric Server", || {
        html! {}
    }),
    ("fa fa-fw fa-comments-o", "Support", || {
        html! {}
    }),
];

impl PvePageConfiguration {
    fn create_menu(&self, _ctx: &Context<Self>) -> Html {
        Column::new()
            .class("pwt-flex-fill pwt-overflow-auto")
            .children(CONFIGS.iter().map(|item| {
                ListTile::new()
                    .interactive(true)
                    .class("pwt-border-bottom")
                    .class("pwt-gap-1")
                    .with_child(html! {<i class={classes!("pwt-font-size-title-large", item.0)}/>})
                    .with_child(item.1)
                    //.onclick(|_| { /* fixme */ })
                    .into()
            }))
            .into()
    }
}

impl Component for PvePageConfiguration {
    type Message = ();
    type Properties = PageConfiguration;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new().title("Configuration"))
            .with_child(self.create_menu(ctx))
            .into()
    }
}

impl Into<VNode> for PageConfiguration {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageConfiguration>(Rc::new(self), None);
        VNode::from(comp)
    }
}
