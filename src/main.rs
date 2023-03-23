mod top_nav_bar;
pub use top_nav_bar::TopNavBar;

mod page_dashboard;
pub use page_dashboard::PageDashboard;

mod page_not_found;
pub use page_not_found::PageNotFound;

use percent_encoding::percent_decode_str;

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew_router::{HashRouter, Routable, Switch};

use pwt::widget::ThemeLoader;
use pwt::touch::PageStack;

use proxmox_yew_comp::{http_login, http_set_auth};
use proxmox_yew_comp::{LoginInfo, ProxmoxProduct};

pub enum Msg {
    Login(LoginInfo),
    //Logout,
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Dashboard,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    let stack = match routes {
        Route::Dashboard => {
            vec![PageDashboard::new().into()]
        }
        Route::NotFound => {
            vec![html! { <PageNotFound/> }]
        }
    };
    PageStack::new(stack).into()
}

struct PveMobileApp {
    login_info: Option<LoginInfo>,
}

impl Component for PveMobileApp {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // set auth info from cookie
        let login_info = LoginInfo::from_cookie(ProxmoxProduct::PVE);
        if let Some(login_info) = &login_info {
            http_set_auth(login_info.clone());
        } else {
            // fixme: show login  page
        }

        Self { login_info }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        ThemeLoader::new(html! {
            <HashRouter>
                <Switch<Route> render={switch} />
            </HashRouter>
        })
        .into()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Login(info) => {
                self.login_info = Some(info);
                let document = web_sys::window().unwrap().document().unwrap();
                let location = document.location().unwrap();
                let _ = location.replace("/");
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
