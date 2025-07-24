use std::rc::Rc;

use yew::virtual_dom::{VComp, VNode};
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;
use pwt::widget::{Column, Fa, List};

use crate::widgets::TopNavBar;

#[derive(Clone, PartialEq, Properties)]
pub struct PageConfiguration {}

use crate::widgets::icon_list_tile;
use crate::Route;

impl PageConfiguration {
    pub fn new() -> Self {
        Self {}
    }
}
pub struct PvePageConfiguration {}

static CONFIGS: &[(&'static str, &'static str, &'static Route)] = &[
    ("asterisk", "Settings", &Route::Settings),
    //("unlock", "Permissions", &Route::Settings),

    /*
    ("server", "Cluster", || {
        html! {}
    }),
    ("gear", "Options", || {
        html! {}
    }),
    ("database", "Storage", || {
        html! {}
    }),
    ("floppy-o", "Backup", || {
        html! {}
    }),
    ("retweet", "Replication", || {
        html! {}
    }),
     ("heartbeat", "High Availability", || {
        html! {}
    }),
    ("certificate", "ACME", || {
        html! {}
    }),
    ("shield", "Firewall", || {
        html! {}
    }),
    ("bar-chart", "Metric Server", || {
        html! {}
    }),
    ("comments-o", "Support", || {
        html! {}
    }),
    */
];

impl PvePageConfiguration {
    fn create_menu(&self, ctx: &Context<Self>) -> Html {
        let navigator = ctx.link().navigator().unwrap();
        List::new(CONFIGS.len() as u64, move |pos| {
            let item = CONFIGS[pos as usize];

            icon_list_tile(
                Fa::new(item.0.to_string()),
                item.1.to_string(),
                None::<&str>,
                None,
            )
            .interactive(true)
            .onclick({
                let navigator = navigator.clone();
                move |_| navigator.push(item.2)
            })
            .into()
        })
        .grid_template_columns("auto 1fr")
        .class("pwt-fit")
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
