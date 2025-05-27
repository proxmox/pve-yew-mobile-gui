use std::rc::Rc;

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::Column;

use crate::widgets::TopNavBar;

use proxmox_yew_comp::LoginPanel;

use proxmox_login::Authentication;

#[derive(Clone, PartialEq, Properties)]
pub struct PageLogin {
    #[prop_or_default]
    pub on_login: Option<Callback<Authentication>>,
}

impl PageLogin {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn on_login(mut self, cb: impl IntoEventCallback<Authentication>) -> Self {
        self.on_login = cb.into_event_callback();
        self
    }
}

pub struct PvePageLogin {}

impl Component for PvePageLogin {
    type Message = ();
    type Properties = PageLogin;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Column::new()
            .class("pwt-flex-fill")
            .class("pwt-viewport")
            .with_child(TopNavBar::new())
            .with_child(
                LoginPanel::new()
                    .mobile(true)
                    .on_login(props.on_login.clone()),
            )
            .into()
    }
}

impl Into<VNode> for PageLogin {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageLogin>(Rc::new(self), None);
        VNode::from(comp)
    }
}
