mod list_tile;
pub use list_tile::*;

mod top_nav_bar;
pub use top_nav_bar::TopNavBar;

mod page_dashboard;
pub use page_dashboard::PageDashboard;

mod page_resources;
pub use page_resources::PageResources;

mod vm_status;
pub use vm_status::PageVmStatus;

mod page_login;
pub use page_login::PageLogin;

mod page_logs;
pub use page_logs::PageLogs;

mod page_configuartion;
pub use page_configuartion::PageConfiguration;

mod page_not_found;
pub use page_not_found::PageNotFound;

use yew_router::{HashRouter, Routable, Switch};
use yew::virtual_dom::Key;

use pwt::prelude::*;
use pwt::touch::{NavigationBar, PageStack};
use pwt::widget::{Column, TabBarItem, ThemeLoader};

use proxmox_yew_comp::{http_login, http_set_auth};
use proxmox_yew_comp::{LoginInfo, ProxmoxProduct};

pub fn goto_location(path: &str) {
    let document = web_sys::window().unwrap().document().unwrap();
    let location = document.location().unwrap();
    let _ = location.replace(&format!("/#{path}"));
}

pub enum Msg {
    Login(LoginInfo),
    //Logout,
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Dashboard,
    #[at("/resources")]
    Resources,
    #[at("/resources/qemu/:vmid")]
    Qemu { vmid: u64 },
    #[at("/logs")]
    Logs,
    #[at("/configuration")]
    Configuration,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    let (active_nav, stack) = match routes {
        Route::Dashboard => {
            (
                "dashboard",
                vec![PageDashboard::new().into()],
            )
        }
        Route::Resources => {
            (
                "resources",
                vec![PageResources::new().into()],
            )
        }
        Route::Qemu { vmid } => {
            (
                "resources",
                vec![
                    PageResources::new().into(),
                    PageVmStatus::new(vmid).into(),
                ],
            )
        }
        Route::Logs => {
            (
                "logs",
                vec![PageLogs::new().into()],
            )
        }
        Route::Configuration => {
            (
                "configuration",
                vec![PageConfiguration::new().into()],
            )
        }
        Route::NotFound => {
            (
                "",
                vec![html! { <PageNotFound/> }],
            )
        }
    };

    let items = vec![
        TabBarItem::new()
            .key("dashboard")
            .icon_class("fa fa-tachometer")
            .on_activate(Callback::from(|_| {
                goto_location("/");
            }))
            .label("Dashboard"),
        TabBarItem::new()
            .key("resources")
            .icon_class("fa fa-book")
            .on_activate(Callback::from(|_| {
                goto_location("/resources");
            }))
            .label("Resources"),
        TabBarItem::new()
            .key("logs")
            .icon_class("fa fa-list")
            .on_activate(Callback::from(|_| {
                goto_location("/logs");
            }))
            .label("Logs"),
        TabBarItem::new()
            .key("configuration")
            .icon_class("fa fa-cogs")
            .on_activate(Callback::from(|_| {
                goto_location("/configuration");
            }))
           .label("Configuration"),
    ];

    let navigation = NavigationBar::new(items)
        .active_item(Key::from(active_nav));

    Column::new()
        .class("pwt-viewport")
        .with_child(PageStack::new(stack))
        .with_child(navigation)
        .into()
}

struct PveMobileApp {
    login_info: Option<LoginInfo>,
}

impl Component for PveMobileApp {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        // set auth info from cookie
        let login_info = LoginInfo::from_cookie(ProxmoxProduct::PVE);
        if let Some(login_info) = &login_info {
            http_set_auth(login_info.clone());
        }
        Self { login_info }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let content: Html = match &self.login_info {
            Some(_info) => {
                html! {
                    <HashRouter>
                        <Switch<Route> render={switch} />
                    </HashRouter>
                }
            }
            None => PageLogin::new()
                .on_login(ctx.link().callback(Msg::Login))
                .into(),
        };

        ThemeLoader::new(content).into()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Login(info) => {
                self.login_info = Some(info);
                goto_location("/");
            }
        }
        true
    }
}

fn main() {
    proxmox_yew_comp::http_setup(proxmox_yew_comp::ProxmoxProduct::PVE);

    pwt::props::set_http_get_method(
        |url| async move { proxmox_yew_comp::http_get(&url, None).await },
    );

    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<PveMobileApp>::new().render();
}
