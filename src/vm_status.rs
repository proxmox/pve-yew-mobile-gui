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
pub struct PageVmStatus {
    vmid: u64,
}

impl PageVmStatus {
    pub fn new(vmid: u64) -> Self {
        Self {
            vmid,
        }
    }
}

pub struct PvePageVmStatus {
}

pub enum Msg {
}

impl Component for PvePageVmStatus {
    type Message = Msg;
    type Properties = PageVmStatus;

    fn create(_ctx: &Context<Self>) -> Self {

        Self {}
    }


    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = Column::new()
            .padding(2)
            .with_child(format!("This is the VmStatus for VM {}", props.vmid));

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
            .with_child(
                TopNavBar::new()
                    .title(format!("VM {}", props.vmid))
                    .back("/resources")
            )
            .with_child(content)
            .with_child(fab)
            .into()
    }
}

impl Into<VNode> for PageVmStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageVmStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}