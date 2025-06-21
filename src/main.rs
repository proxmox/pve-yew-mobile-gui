pub mod widgets;

pub mod pages;
use pages::{
    PageConfiguration, PageContainerStatus, PageDashboard, PageLogin, PageNodeStatus, PageNotFound,
    PageResources, PageSettings, PageStorageStatus, PageTaskStatus, PageTasks, PageVmStatus,
    ResourceFilter,
};

use yew::virtual_dom::Key;
use yew_router::history::{AnyHistory, History};
use yew_router::scope_ext::RouterScopeExt;
use yew_router::Routable;

use pwt::prelude::*;
use pwt::state::LanguageInfo;
use pwt::touch::{MaterialApp, MaterialAppRouteContext, NavigationBar};
use pwt::widget::{Column, TabBarItem};

use proxmox_login::Authentication;

use proxmox_yew_comp::{
    authentication_from_cookie, http_set_auth, percent_encoding::percent_encode_component,
    register_auth_observer, AuthObserver,
};

pub enum Msg {
    Login(Authentication),
    Logout,
}

#[derive(Clone, Debug, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Dashboard,
    #[at("/settings")]
    Settings,
    #[at("/resources")]
    Resources,

    #[at("/resources/qemu/:nodename/:vmid")]
    Qemu { vmid: u32, nodename: String },
    #[at("/resources/qemu/:nodename/:vmid/tasks")]
    QemuTasks { vmid: u32, nodename: String },
    #[at("/resources/qemu/:nodename/:vmid/tasks/:upid/:endtime")]
    QemuTaskStatus {
        vmid: u32,
        nodename: String,
        upid: String,
        endtime: i64,
    },

    #[at("/resources/lxc/:nodename/:vmid")]
    Lxc { vmid: u32, nodename: String },
    #[at("/resources/node/:nodename")]
    Node { nodename: String },
    #[at("/resources/node/:nodename/tasks")]
    NodeTasks { nodename: String },
    #[at("/resources/node/:nodename/tasks/:upid/:endtime")]
    NodeTasksStatus {
        nodename: String,
        upid: String,
        endtime: i64,
    },
    #[at("/resources/storage/:name")]
    Storage { name: String },
    // #[at("/logs")]
    // Logs,
    #[at("/configuration")]
    Configuration,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn main_nav_page(active_nav: &str, content: impl Into<Html>, history: &AnyHistory) -> Html {
    let nav_items = vec![
        TabBarItem::new()
            .key("dashboard")
            .icon_class("fa fa-tachometer")
            .on_activate({
                let history = history.clone();
                move |_| {
                    history.push(&Route::Dashboard.to_path());
                }
            })
            .label("Dashboard"),
        TabBarItem::new()
            .key("resources")
            .icon_class("fa fa-book")
            .on_activate({
                let history = history.clone();
                move |_| {
                    history.push(&Route::Resources.to_path());
                }
            })
            .label("Resources"),
        TabBarItem::new()
            .key("configuration")
            .icon_class("fa fa-cogs")
            .on_activate({
                let history = history.clone();
                move |_| {
                    history.push(&Route::Configuration.to_path());
                }
            })
            .label("Configuration"),
    ];

    let navigation = NavigationBar::new(nav_items).active(Key::from(active_nav));
    Column::new()
        .class("pwt-viewport")
        .with_child(content)
        .with_child(navigation)
        .into()
}

fn switch(context: &MaterialAppRouteContext, path: &str) -> Vec<Html> {
    let route = Route::recognize(&path).unwrap();
    let active_nav = if path.starts_with("/resources") {
        "resources"
    } else if path.starts_with("/configuration") {
        "configuration"
    } else {
        "dashboard"
    };
    switch_route(context, route, active_nav)
}

fn switch_route(context: &MaterialAppRouteContext, route: Route, active_nav: &str) -> Vec<Html> {
    let history = &context.history;

    let (mut stack, content) = match route {
        Route::Dashboard => (
            vec![],
            main_nav_page(active_nav, PageDashboard::new(), history),
        ),
        Route::Settings => (
            switch_route(context, Route::Dashboard, active_nav),
            PageSettings::new().into(),
        ),
        Route::Resources => (
            vec![],
            main_nav_page(active_nav, PageResources::new(), history),
        ),
        Route::Qemu { vmid, nodename } => (
            switch_route(context, Route::Resources, active_nav),
            PageVmStatus::new(nodename, vmid).into(),
        ),
        Route::QemuTasks { vmid, nodename } => (
            switch_route(
                context,
                Route::Qemu {
                    vmid: vmid.clone(),
                    nodename: nodename.clone(),
                },
                active_nav,
            ),
            PageTasks::new(format!(
                "/nodes/{}/tasks?vmid={vmid}",
                percent_encode_component(&nodename),
            ))
            .title(format!("VM {vmid}"))
            .back(format!(
                "/resources/qemu/{}/{}",
                percent_encode_component(&nodename),
                vmid
            ))
            .on_show_task({
                let history = history.clone();
                move |(upid, endtime): (String, Option<i64>)| {
                    history.push(
                        &Route::QemuTaskStatus {
                            vmid,
                            nodename: nodename.clone(),
                            upid,
                            endtime: endtime.unwrap_or(0),
                        }
                        .to_path(),
                    );
                }
            })
            .into(),
        ),
        Route::QemuTaskStatus {
            vmid,
            nodename,
            upid,
            endtime,
        } => (
            switch_route(
                context,
                Route::QemuTasks {
                    vmid: vmid.clone(),
                    nodename: nodename.clone(),
                },
                active_nav,
            ),
            PageTaskStatus::new(
                format!("/nodes/{}/tasks", percent_encode_component(&nodename)),
                upid,
            )
            .endtime(endtime)
            .back(format!(
                "/resources/qemu/{}/{}/tasks",
                percent_encode_component(&nodename),
                vmid
            ))
            .into(),
        ),
        Route::Lxc { nodename, vmid } => (
            switch_route(context, Route::Resources, active_nav),
            PageContainerStatus::new(nodename, vmid).into(),
        ),
        Route::Node { nodename } => (
            switch_route(context, Route::Resources, active_nav),
            PageNodeStatus::new(nodename).into(),
        ),
        Route::NodeTasks { nodename } => (
            switch_route(
                context,
                Route::Node {
                    nodename: nodename.clone(),
                },
                active_nav,
            ),
            PageTasks::new(format!(
                "/nodes/{}/tasks",
                percent_encode_component(&nodename),
            ))
            .title(format!("Node {nodename}"))
            .back(format!(
                "/resources/node/{}",
                percent_encode_component(&nodename),
            ))
            .on_show_task({
                let history = history.clone();
                move |(upid, endtime): (String, Option<i64>)| {
                    history.push(
                        &Route::NodeTasksStatus {
                            nodename: nodename.clone(),
                            upid,
                            endtime: endtime.unwrap_or(0),
                        }
                        .to_path(),
                    );
                }
            })
            .into(),
        ),
        Route::NodeTasksStatus {
            nodename,
            upid,
            endtime,
        } => (
            switch_route(
                context,
                Route::NodeTasks {
                    nodename: nodename.clone(),
                },
                active_nav,
            ),
            PageTaskStatus::new(
                format!("/nodes/{}/tasks", percent_encode_component(&nodename)),
                upid,
            )
            .endtime(endtime)
            .back(format!(
                "/resources/node/{}/tasks",
                percent_encode_component(&nodename),
            ))
            .into(),
        ),
        Route::Storage { name } => (
            switch_route(context, Route::Resources, active_nav),
            PageStorageStatus::new(name).into(),
        ),
        Route::Configuration => (
            vec![],
            main_nav_page(active_nav, PageConfiguration::new(), history),
        ),
        Route::NotFound => (vec![], html! { <PageNotFound/> }),
    };

    stack.push(content);
    stack
}

/*
fn switch(context: &MaterialAppRouteContext, full_path: &str) -> Vec<Html> {
    log::info!("SWITCH {full_path}");
    let history = &context.history;

    let mut components: Vec<String> = full_path
        .trim_matches('/')
        .split("/")
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    log::info!("COMPS {components:?}");

    let active_nav = match components.get(0).map(|s| s.as_str()) {
        Some("resources") => "resources",
        Some("configuration") => "configuration",
        _ => "dashboard",
    };

    let mut stack = Vec::new();

    if components.is_empty() {
        stack.push(switch_single_page(context, Route::recognize("/").unwrap()));
    } else {
        let not_found_route = Route::not_found_route();

        loop {
            if components.is_empty() {
                break;
            }
            let path = format!("/{}", components.join("/"));
            if let Some(route) = Route::recognize(&path) {
                if let Some(not_found_route) = &not_found_route {
                    if route == *not_found_route {
                        components.pop();
                        continue;
                    }
                }
                log::info!("PUSH {path} {route:?}");

                let page = switch_single_page(context, route);
                stack.push(page);
            }
            components.pop();
        }

        stack.reverse();
    }

    //main_nav_page(active_nav, content);

    stack
}
    */

struct PveMobileApp {
    _auth_observer: AuthObserver,
    login_info: Option<Authentication>,
}

impl Component for PveMobileApp {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // set auth info from cookie
        let login_info = authentication_from_cookie(&proxmox_yew_comp::ExistingProduct::PVE);
        if let Some(login_info) = &login_info {
            http_set_auth(login_info.clone());
        }

        let _auth_observer = register_auth_observer(ctx.link().callback(|_| Msg::Logout));

        Self {
            login_info,
            _auth_observer,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let auth = self.login_info.is_some();
        let link = ctx.link().clone();

        let render = move |context: &MaterialAppRouteContext, path: &str| {
            if auth {
                switch(context, path)
            } else {
                return vec![PageLogin::new().on_login(link.callback(Msg::Login)).into()];
            }
        };

        MaterialApp::new(render).into()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let navigator = ctx.link().navigator().clone();
        match msg {
            Msg::Login(info) => {
                self.login_info = Some(info);
                if let Some(navigator) = &navigator {
                    navigator.push(&Route::Dashboard);
                }
            }
            Msg::Logout => {
                self.login_info = None;
                if let Some(navigator) = &navigator {
                    navigator.push(&Route::Dashboard);
                }
            }
        }
        true
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    proxmox_yew_comp::http_setup(&proxmox_yew_comp::ExistingProduct::PVE);

    pwt::props::set_http_get_method(
        |url| async move { proxmox_yew_comp::http_get(&url, None).await },
    );

    pwt::state::set_available_themes(&["Material", "Crisp"]);

    pwt::state::set_available_languages(vec![LanguageInfo::new(
        "en",
        "English",
        gettext_noop("English"),
    )]);
    yew::Renderer::<PveMobileApp>::new().render();
}
