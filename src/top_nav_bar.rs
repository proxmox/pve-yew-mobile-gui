use std::rc::Rc;
use std::ops::Deref;

use pwt::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use pwt::state::{Theme, ThemeObserver};
use pwt::widget::{ActionIcon, Column, ThemeModeSelector, Row};
use pwt::widget::menu::{Menu, MenuItem, MenuButton};

#[derive(Clone, PartialEq, Properties)]
pub struct TopNavBar {
    pub title: Option<AttrValue>,
    pub subtitle: Option<AttrValue>,

    pub back: Option<AttrValue>,

    #[prop_or_default]
    pub class: Classes,

    pub on_logout: Option<Callback<MouseEvent>>,
}

impl TopNavBar {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to add a html class
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
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

        let menu = Menu::new()
            .with_item(MenuItem::new("Submenu1 Item1"))
            .with_item(MenuItem::new("Submenu1 Item2"));

        let button_group = Row::new()
            .gap(1)
            //.with_child(HelpButton::new().class("neutral"))
            .with_child(ThemeModeSelector::new().class("neutral"))
            .with_child(
                MenuButton::new("").class("circle").icon_class("fa fa-bars").menu(menu)
            )
            /*
            .with_child(
                Button::new("Logout")
                    .class("secondary")
                    .onclick(props.on_logout.clone())
            )*/
            ;

        let back_or_logo = if let Some(back) = &props.back {
            let back = back.to_owned();
            ActionIcon::new("fa fa-angle-left")
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
                <span class="pwt-font-title-large">{title}</span>
            });

        let subtitle = if props.title.is_none() {
            Some("Virtual Environment")
        } else {
            props.subtitle.as_deref()
        };

        if let Some(subtitle) = subtitle {
            text_block.add_child(html!{
                <span class="pwt-font-title-small">{subtitle}</span>
            });
        }

        Row::new()
            .gap(1)
            .attribute("role", "banner")
            .attribute("aria-label", "Proxmox VE")
            .class("pwt-navbar")
            .class("pwt-justify-content-space-between pwt-align-items-center")
            //.class("pwt-border-bottom")
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
