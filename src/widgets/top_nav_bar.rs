use std::ops::Deref;
use std::rc::Rc;

use pwt::prelude::*;
use yew::html::IntoPropValue;
use yew::virtual_dom::{VComp, VNode};

use pwt::impl_class_prop_builder;
use pwt::state::{Theme, ThemeObserver};
use pwt::widget::menu::{Menu, MenuButton, MenuItem};
use pwt::widget::{ActionIcon, Column, Row, ThemeModeSelector};

use pwt_macros::builder;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct TopNavBar {
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub title: Option<AttrValue>,

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub subtitle: Option<AttrValue>,

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub back: Option<AttrValue>,

    #[prop_or_default]
    pub class: Classes,
}

impl TopNavBar {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    impl_class_prop_builder!();
}

pub enum Msg {
    ThemeChanged((Theme, /* dark_mode */ bool)),
    Logout,
}

pub struct PveTopNavBar {
    _theme_observer: ThemeObserver,
    dark_mode: bool,
}

impl Component for PveTopNavBar {
    type Message = Msg;
    type Properties = TopNavBar;

    fn create(ctx: &Context<Self>) -> Self {
        let _theme_observer = ThemeObserver::new(ctx.link().callback(Msg::ThemeChanged));
        let dark_mode = _theme_observer.dark_mode();
        Self {
            _theme_observer,
            dark_mode,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ThemeChanged((_theme, dark_mode)) => {
                self.dark_mode = dark_mode;
                true
            }
            Msg::Logout => {
                proxmox_yew_comp::http_clear_auth();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let menu = Menu::new().with_item(
            MenuItem::new("Logout")
                .icon_class("fa fa-sign-out")
                .on_select(ctx.link().callback(|_| Msg::Logout)),
        );

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
                .class("pwt-scheme-neutral")
                .on_activate({
                    move |_| {
                        crate::goto_location(&back);
                    }
                })
                .into()
        } else {
            let src = if self.dark_mode {
                "/images/proxmox_logo_icon_black.png"
            } else {
                "/images/proxmox_logo_icon_white.png"
            };
            html! { <img class="pwt-navbar-brand" {src} alt="Proxmox logo"/> }
        };

        let title = match &props.title {
            Some(text) => text.deref(),
            None => "Proxmox",
        };

        let mut text_block = Column::new().with_child(html! {
            <span class="pwt-font-title-large">{title}</span>
        });

        let subtitle = if props.title.is_none() {
            Some("Virtual Environment")
        } else {
            props.subtitle.as_deref()
        };

        if let Some(subtitle) = subtitle {
            text_block.add_child(html! {
                <span class="pwt-font-title-small">{subtitle}</span>
            });
        }

        Row::new()
            .gap(1)
            .attribute("role", "banner")
            .attribute("aria-label", "Proxmox VE")
            .class("pwt-navbar")
            .class("pwt-bg-color-primary pwt-color-on-primary")
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
        let comp = VComp::new::<PveTopNavBar>(Rc::new(self), None);
        VNode::from(comp)
    }
}
