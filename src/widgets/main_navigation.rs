use std::rc::Rc;

use yew::virtual_dom::{Key, VComp, VNode};
use yew_router::prelude::LocationHandle;
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;

use pwt::touch::NavigationBar;
use pwt::widget::{Column, TabBarItem};

#[derive(Clone, PartialEq, Properties)]
pub struct MainNavigation {}

impl MainNavigation {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub struct PveMainNavigation {
    _location_handle: LocationHandle,
}

impl Component for PveMainNavigation {
    type Message = ();
    type Properties = MainNavigation;

    fn create(ctx: &Context<Self>) -> Self {
        let _location_handle = ctx
            .link()
            .add_location_listener(ctx.link().callback(|_| ()))
            .unwrap();
        Self { _location_handle }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let location = ctx.link().location().unwrap();
        let path = location.path();

        let (active_nav, content): (_, Html) = if path.starts_with("/resources") {
            ("resources", crate::pages::PageResources::new().into())
        } else if path.starts_with("/configuration") {
            (
                "configuration",
                crate::pages::PageConfiguration::new().into(),
            )
        } else {
            ("dashboard", crate::pages::PageDashboard::new().into())
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
                        navigator.push(&crate::Route::Resources);
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
