use std::rc::Rc;

use yew::virtual_dom::{Key, VComp, VNode};
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;

use pwt::touch::NavigationBar;
use pwt::widget::{Column, TabBarItem};

use crate::pages::ResourceFilter;

#[derive(Copy, Clone, PartialEq)]
pub enum MainNavigationSelection {
    Dashboard,
    Resources,
    Configuration,
}

#[derive(Clone, PartialEq, Properties)]
pub struct MainNavigation {
    pub active_nav: MainNavigationSelection,
}

impl MainNavigation {
    pub fn new(active_nav: MainNavigationSelection) -> Self {
        yew::props!(Self { active_nav })
    }
}

pub struct PveMainNavigation {}

impl Component for PveMainNavigation {
    type Message = ();
    type Properties = MainNavigation;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (active_nav, content): (_, Html) = match props.active_nav {
            MainNavigationSelection::Configuration => (
                "configuration",
                crate::pages::PageConfiguration::new().into(),
            ),
            MainNavigationSelection::Dashboard => {
                ("dashboard", crate::pages::PageDashboard::new().into())
            }
            MainNavigationSelection::Resources => {
                ("resources", crate::pages::PageResources::new().into())
            }
        };

        let nav_items = vec![
            TabBarItem::new()
                .key("dashboard")
                .icon_class("fa fa-tachometer")
                .on_activate({
                    let navigator = ctx.link().navigator().unwrap();
                    move |_| {
                        navigator.push(&crate::Route::Dashboard);
                    }
                })
                .label("Dashboard"),
            TabBarItem::new()
                .key("resources")
                .icon_class("fa fa-book")
                .on_activate({
                    let navigator = ctx.link().navigator().unwrap();
                    move |_| {
                        // clear filter state? not sure if this is good or bad
                        let filter = ResourceFilter::default();
                        navigator.push_with_state(&crate::Route::Resources, filter);
                        // navigator.push(&crate::Route::Resources);
                    }
                })
                .label("Resources"),
            TabBarItem::new()
                .key("configuration")
                .icon_class("fa fa-cogs")
                .on_activate({
                    let navigator = ctx.link().navigator().unwrap();
                    move |_| {
                        navigator.push(&crate::Route::Configuration);
                    }
                })
                .label("Configuration"),
        ];

        let navigation = NavigationBar::new(nav_items).active(Key::from(active_nav));
        Column::new()
            .class("pwt-fit")
            .with_child(content)
            .with_child(navigation)
            .into()
    }
}

impl From<MainNavigation> for VNode {
    fn from(props: MainNavigation) -> Self {
        let comp = VComp::new::<PveMainNavigation>(Rc::new(props), None);
        VNode::from(comp)
    }
}
