use std::rc::Rc;
use std::ops::Deref;

use pwt::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use pwt::state::{Theme, ThemeObserver};
use pwt::widget::{ActionIcon, Column, ThemeModeSelector, Row};

#[derive(Clone, PartialEq, Properties)]
pub struct TopNavBar {
    pub title: Option<AttrValue>,
    pub subtitle: Option<AttrValue>,

    pub back: Option<AttrValue>,

    pub on_logout: Option<Callback<MouseEvent>>,
}

impl TopNavBar {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn title(mut self, title: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.title = title.into_prop_value();
        self
    }

    pub fn subtitle(mut self, subtitle: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.subtitle = subtitle.into_prop_value();
        self
    }

    pub fn back(mut self, link: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.back = link.into_prop_value();
        self
    }

    pub fn on_logout(mut self, cb: impl IntoEventCallback<MouseEvent>) -> Self {
        self.on_logout = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    ThemeChanged((Theme, /* dark_mode */ bool)),
}

pub struct PmgTopNavBar {
    _theme_observer: ThemeObserver,
    dark_mode: bool,
}



impl Component for PmgTopNavBar {
    type Message = Msg;
    type Properties = TopNavBar;

    fn create(ctx: &Context<Self>) -> Self {
        let _theme_observer = ThemeObserver::new(ctx.link().callback(Msg::ThemeChanged));
        let dark_mode = _theme_observer.dark_mode();
        Self {
            _theme_observer,
            dark_mode ,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ThemeChanged((_theme, dark_mode)) => {
                self.dark_mode = dark_mode;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let button_group = Row::new()
            .gap(1)
            //.with_child(HelpButton::new().class("neutral"))
            .with_child(ThemeModeSelector::new().class("neutral"))
            /*
            .with_child(
                Button::new("Logout")
                    .class("secondary")
                    .onclick(props.on_logout.clone())
            )*/
            ;

        let back_or_logo = if let Some(back) = &props.back {
            let back = back.to_owned();
            ActionIcon::new("fa fa-arrow-left")
                .class("pwt-font-size-headline-small")
                .class("pwt-color-primary")
                .on_activate({
                    move |_| {
                        crate::goto_location(&back);
                    }
                })
                .into()
        } else {
            let src = if self.dark_mode {
                "/proxmox_logo_icon_white.png"
            } else {
                "/proxmox_logo_icon_black.png"
            };
            html!{ <img class="pwt-navbar-brand" {src} alt="Proxmox logo"/> }
        };

        let title = match &props.title {
            Some(text) => text.deref(),
            None => "Proxmox",
        };

        let mut text_block = Column::new()
            .with_child(html!{
                <span class="pwt-ps-1 pwt-font-title-large">{title}</span>
            });

        let subtitle = if props.title.is_none() {
            Some("Virtual Environment")
        } else {
            props.subtitle.as_deref()
        };

        if let Some(subtitle) = subtitle {
            text_block.add_child(html!{
                <span class="pwt-ps-1 pwt-font-title-small">{subtitle}</span>
            });
        }

        Row::new()
            .attribute("role", "banner")
            .attribute("aria-label", "Proxmox VE")
            .class("pwt-navbar")
            .class("pwt-justify-content-space-between pwt-align-items-center")
            .class("pwt-border-bottom")
            .padding(1)
            .with_child(back_or_logo)
            .with_child(text_block)
            .with_flex_spacer()
            .with_child(button_group)
            .into()
    }
}

impl Into<VNode> for TopNavBar {
    fn into(self) -> VNode {
        let comp = VComp::new::<PmgTopNavBar>(Rc::new(self), None);
        VNode::from(comp)
    }
}
