use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Button, Column, Container, Row};

use crate::widgets::TopNavBar;

use proxmox_yew_comp::LoginPanel;

use proxmox_login::Authentication;

use pwt_macros::builder;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct PageLogin {
    #[builder_cb(IntoEventCallback, into_event_callback, Authentication)]
    #[prop_or_default]
    pub on_login: Option<Callback<Authentication>>,

    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub consent_text: Option<AttrValue>,
}

impl PageLogin {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub enum Msg {
    Consent,
}

pub struct PvePageLogin {
    consent: bool,
}

impl Component for PvePageLogin {
    type Message = Msg;
    type Properties = PageLogin;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { consent: false }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Consent => {
                self.consent = true;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let consent_text = if self.consent {
            None
        } else {
            props.consent_text.as_ref().filter(|t| !t.is_empty())
        };

        let content: Html = if let Some(consent_text) = consent_text {
            let card = crate::widgets::standard_card(tr!("Consent"), ())
                .class("pwt-scheme-neutral")
                .with_child(
                    Column::new()
                        .padding(2)
                        .gap(2)
                        .with_child(Container::from_tag("p").with_child(consent_text))
                        .with_child(
                            Row::new().with_flex_spacer().with_child(
                                Button::new(tr!("OK"))
                                    .on_activate(ctx.link().callback(|_| Msg::Consent)),
                            ),
                        ),
                );
            Column::new()
                .class(pwt::css::FlexFit)
                .class(pwt::css::JustifyContent::Center)
                .padding(2)
                .with_child(card)
                .into()
        } else {
            LoginPanel::new()
                .mobile(true)
                .on_login(props.on_login.clone())
                .into()
        };

        Column::new()
            .class("pwt-flex-fill")
            .class("pwt-viewport")
            .with_child(TopNavBar::new())
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageLogin {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageLogin>(Rc::new(self), None);
        VNode::from(comp)
    }
}
