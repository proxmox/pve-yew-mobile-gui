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

use crate::{ReloadController, Route, SpamList, TopNavBar};

#[derive(Clone, PartialEq, Properties)]
pub struct PageSpamList {
    reload_controller: ReloadController,
}

impl PageSpamList {
    pub fn new(reload_controller: ReloadController) -> Self {
        Self {
            reload_controller,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ViewState {
    Normal,
    ShowDialog,
}
pub struct PmgPageSpamList {
    state: ViewState,
    start_date: f64,
    end_date: f64,
    form_context: FormContext,
}

pub enum Msg {
    Preview(String),
    ShowDialog,
    CloseDialog,
    ApplyDate,
}

fn epoch_to_date_string(epoch: f64) -> String {
    let start_date = Date::new(&JsValue::from_f64(epoch));
    format!(
        "{:04}-{:02}-{:02}",
        start_date.get_full_year(),
        start_date.get_month() + 1,
        start_date.get_date(),
    )
}
impl PmgPageSpamList {

    fn date_range_form(&self, ctx: &Context<Self>) -> Html {
        let start_date = epoch_to_date_string(self.start_date);
        let end_date = epoch_to_date_string(self.end_date);

        let panel = Column::new()
            .padding(2)
            .gap(1)
            //.attribute("style", "min-width:400px;min-height:300px;")
            .class("pwt-flex-fill")
            .with_child("From:")
            .with_child(
                Field::new()
                    .name("from")
                    .default(start_date)
                    .input_type("date")
            )
            .with_child("To:")
            .with_child(
                Field::new()
                    .name("to")
                    .default(end_date)
                    .input_type("date")
            )
            .with_child(
                Row::new()
                    .class("pwt-pt-2")
                    .with_flex_spacer()
                    .with_child(
                        Button::new("Apply")
                            .class("pwt-scheme-primary")
                            .onclick(ctx.link().callback(|_| Msg::ApplyDate))
                    )
            );

        Form::new()
            .form_context(self.form_context.clone())
            .with_child(panel)
            .into()
    }
}

impl Component for PmgPageSpamList {
    type Message = Msg;
    type Properties = PageSpamList;

    fn create(_ctx: &Context<Self>) -> Self {
        let start_date = js_sys::Date::new_0();
        start_date.set_hours(0);
        start_date.set_minutes(0);
        start_date.set_seconds(0);
        start_date.set_milliseconds(0);

        let mut start_date = start_date.get_time();
        let end_date = start_date + 24.0*3600000.0;
        start_date = end_date - 7.0*24.0*3600000.0;

        Self {
            state: ViewState::Normal,
            start_date,
            end_date,
            form_context: FormContext::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ShowDialog => {
                self.state = ViewState::ShowDialog;
                true
            }
            Msg::CloseDialog => {
                self.state = ViewState::Normal;
                true
            }
            Msg::ApplyDate => {
                self.state = ViewState::Normal;

                let start = self.form_context.read().get_field_value("from").unwrap();
                self.start_date = Date::parse(start.as_str().unwrap());
                let end = self.form_context.read().get_field_value("to").unwrap();
                self.end_date = Date::parse(end.as_str().unwrap());

                true
            }
            Msg::Preview(id) => {
                //log::info!("Preview {id}");
                let navigator = ctx.link().navigator().unwrap();
                navigator.push(&Route::ViewMail { id: id.clone() });
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let content = SpamList::new(props.reload_controller.clone())
            .starttime((self.start_date / 1000.0) as u64)
            .endtime((self.end_date / 1000.0) as u64)
            .on_preview(ctx.link().callback(|id| Msg::Preview(id)));

        let dialog = (self.state == ViewState::ShowDialog).then(||  {
            Dialog::new("Select Date")
                .with_child(self.date_range_form(ctx))
                .on_close(ctx.link().callback(|_| Msg::CloseDialog))
        });

        let fab = Container::new()
            .class("pwt-position-fixed")
            .class("pwt-right-2 pwt-bottom-4")
            .with_child(
                Fab::new("fa fa-calendar")
                    .class("pwt-scheme-primary")
                    .on_click(ctx.link().callback(|_| Msg::ShowDialog))
            );

        Column::new()
            .class("pwt-viewport")
            .with_child(TopNavBar::new())
            .with_child(content)
            .with_child(fab)
            .with_optional_child(dialog)
            .into()
    }
}

impl Into<VNode> for PageSpamList {
    fn into(self) -> VNode {
        let comp = VComp::new::<PmgPageSpamList>(Rc::new(self), None);
        VNode::from(comp)
    }
}