pub mod widgets;
pub use widgets::{MainNavigation, MainNavigationSelection};

pub mod pages;
use pages::{
    PageContainerStatus, PageLogin, PageNodeStatus, PageNodeTasks, PageNotFound, PageQemuTasks,
    PageSettings, PageStorageStatus, PageTaskStatus, PageVmStatus,
};

use yew_router::scope_ext::RouterScopeExt;
use yew_router::Routable;

use pwt::prelude::*;
use pwt::state::LanguageInfo;
use pwt::touch::MaterialApp;

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
    NodeTaskStatus {
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

fn switch(path: &str) -> Vec<Html> {
    let route = Route::recognize(&path).unwrap();
    switch_route(route)
}

// Warning: Do not define/use callbacks inside the route switch, because
// that triggers change detection in the PageStack (callbacks are never equal)
fn switch_route(route: Route) -> Vec<Html> {
    let (mut stack, content) = match route {
        Route::Dashboard => (
            vec![],
            MainNavigation::new(MainNavigationSelection::Dashboard).into(),
        ),
        Route::Configuration => (
            vec![],
            MainNavigation::new(MainNavigationSelection::Configuration).into(),
        ),
        Route::Resources => (
            vec![],
            MainNavigation::new(MainNavigationSelection::Resources).into(),
        ),

        Route::Settings => (switch_route(Route::Dashboard), PageSettings::new().into()),
        Route::Qemu { vmid, nodename } => (
            switch_route(Route::Resources),
            PageVmStatus::new(nodename, vmid).into(),
        ),
        Route::QemuTasks { vmid, nodename } => (
            switch_route(Route::Qemu {
                vmid: vmid.clone(),
                nodename: nodename.clone(),
            }),
            PageQemuTasks::new(nodename, vmid).into(),
        ),
        Route::QemuTaskStatus {
            vmid,
            nodename,
            upid,
            endtime,
        } => (
            switch_route(Route::QemuTasks {
                vmid: vmid.clone(),
                nodename: nodename.clone(),
            }),
            PageTaskStatus::new(
                format!("/nodes/{}/tasks", percent_encode_component(&nodename)),
                upid,
            )
            .endtime(endtime)
            .into(),
        ),
        Route::Lxc { nodename, vmid } => (
            switch_route(Route::Resources),
            PageContainerStatus::new(nodename, vmid).into(),
        ),
        Route::Node { nodename } => (
            switch_route(Route::Resources),
            PageNodeStatus::new(nodename).into(),
        ),
        Route::NodeTasks { nodename } => (
            switch_route(Route::Node {
                nodename: nodename.clone(),
            }),
            PageNodeTasks::new(nodename).into(),
        ),
        Route::NodeTaskStatus {
            nodename,
            upid,
            endtime,
        } => (
            switch_route(Route::NodeTasks {
                nodename: nodename.clone(),
            }),
            PageTaskStatus::new(
                format!("/nodes/{}/tasks", percent_encode_component(&nodename)),
                upid,
            )
            .endtime(endtime)
            .into(),
        ),
        Route::Storage { name } => (
            switch_route(Route::Resources),
            PageStorageStatus::new(name).into(),
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

        let render = move |path: &str| {
            if auth {
                switch(path)
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
