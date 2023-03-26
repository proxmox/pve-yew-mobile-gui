use std::rc::Rc;

use js_sys::Date;
use wasm_bindgen::JsValue;

use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::{Fab};
use pwt::widget::{Button, Column, Container, Dialog, Row};
use pwt::widget::form::{Field, Form, FormContext};

use crate::{Route, TopNavBar};

#[derive(Clone, PartialEq, Properties)]
pub struct PageResources {
}

impl PageResources {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct PvePageResources {
}

pub enum Msg {
}

impl Component for PvePageResources {
    type Message = Msg;
    type Properties = PageResources;

    fn create(_ctx: &Context<Self>) -> Self {

        Self {}
    }


    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = Column::new()
            .padding(2)
            .with_child("This is the Resources page.");

        let fab = Container::new()
            .class("pwt-position-absolute")
            .class("pwt-right-2 pwt-bottom-2")
            .with_child(
                Fab::new("fa fa-calendar")
                    .class("pwt-scheme-primary")
                    //.on_click(ctx.link().callback(|_| Msg::ShowDialog))
            );

        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new())
            .with_child(content)
            .with_child(fab)
            .into()
    }
}

impl Into<VNode> for PageResources {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageResources>(Rc::new(self), None);
        VNode::from(comp)
    }
}