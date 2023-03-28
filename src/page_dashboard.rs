use std::rc::Rc;

use js_sys::Date;
use wasm_bindgen::JsValue;

use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::{Fab};
use pwt::widget::{Button, Card, Column, Container, MiniScroll, Panel, Row};
use pwt::widget::form::{Field, Form, FormContext};

use crate::{Route, TopNavBar};

#[derive(Clone, PartialEq, Properties)]
pub struct PageDashboard {
}

impl PageDashboard {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct PvePageDashboard {
}

pub enum Msg {
}

impl PvePageDashboard {
    fn create_tab_bar(&self, ctx: &Context<Self>) -> Html {
        let mut content = Row::new()
        .padding_y(1)
        .gap(2)
        .with_child(
            Button::new("Subscription")
        )
        .with_child(
            Button::new("Virtual Machines")
                .icon_class("fa fa-desktop")
        )
        .with_child(
            Button::new("Containers")
                .icon_class("fa fa-cube")
        );

        MiniScroll::new(content).into()
    }

    fn create_analytics_card(&self, ctx: &Context<Self>) -> Html {
        Card::new()
            .padding(0)
            .with_child(html!{
                <div class="pwt-p-2 pwt-border-bottom">
                    <div class="pwt-font-size-title-large">{"Analytics"}</div>
                    <div class="pwt-font-size-title-small">{"Usage acress all online nodes."}</div>
                </div>
            })
            .with_child(html!{
                <div class="pwt-p-2 pwt-border-bottom">{"CPU"}</div>
            })
            .with_child(html!{
                <div class="pwt-p-2">{"Memory"}</div>
            })
           .into()
    }

    fn create_nodes_card(&self, ctx: &Context<Self>) -> Html {
        Card::new()
            .padding(0)
            .with_child(html!{
                <div class="pwt-p-2 pwt-border-bottom">
                    <div class="pwt-font-size-title-large">{"Nodes"}</div>
                </div>
            })
            .with_child(html!{
                <div class="pwt-p-2">{"Nodes..."}</div>
            })
            .into()
        }

        fn create_guests_card(&self, ctx: &Context<Self>) -> Html {
            Card::new()
                .padding(0)
                .with_child(html!{
                    <div class="pwt-p-2 pwt-border-bottom">
                        <div class="pwt-font-size-title-large">{"Guests"}</div>
                    </div>
                })
                .with_child(html!{
                    <div class="pwt-p-2">{"Guests..."}</div>
                })
                .into()
            }

}

impl Component for PvePageDashboard {
    type Message = Msg;
    type Properties = PageDashboard;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }


    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = Column::new()
            .padding(2)
            .gap(2)
            .with_child(self.create_tab_bar(ctx))
            .with_child(self.create_analytics_card(ctx))
            .with_child(self.create_nodes_card(ctx))
            .with_child(self.create_guests_card(ctx));

        /*
        let fab = Container::new()
            .class("pwt-position-absolute")
            .class("pwt-right-2 pwt-bottom-2")
            .with_child(
                Fab::new("fa fa-calendar")
                    .class("pwt-scheme-primary")
                    //.on_click(ctx.link().callback(|_| Msg::ShowDialog))
            );
        */

        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new())
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageDashboard {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageDashboard>(Rc::new(self), None);
        VNode::from(comp)
    }
}