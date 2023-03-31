use std::rc::Rc;

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, Mask};
use pwt::widget::form::{Field, Form, FormContext, SubmitButton};

use crate::TopNavBar;

use proxmox_yew_comp::{RealmSelector, LoginInfo};

#[derive(Clone, PartialEq, Properties)]
pub struct PageLogin {
    pub on_login: Option<Callback<LoginInfo>>,
}

impl PageLogin {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn on_login(mut self, cb: impl IntoEventCallback<LoginInfo>) -> Self {
        self.on_login = cb.into_event_callback();
        self
    }
}

pub struct PvePageLogin {
    loading: bool,
    login_error: Option<String>,
    form_ctx: FormContext,
}

pub enum Msg {
    FormDataChange,
    Submit,
    Login,
    LoginError(String),
}

impl PvePageLogin {

    fn login_form(&self, ctx: &Context<Self>) -> Html {

        let panel = Column::new()
            .attribute("style", "min-width:300px;")
            .gap(2)
            .with_child(
                Field::new()
                    .name("username")
                    .placeholder("Username")
                    .required(true)
                    .autofocus(true)
            )
            .with_child(
                Field::new()
                    .name("password")
                    .placeholder("Password")
                    .required(true)
                    .input_type("password")
            )
            .with_child(
                RealmSelector::new()
                    .name("realm"),
            )
            .with_optional_child(self.login_error.as_ref().map(|msg| {
                pwt::widget::error_message(msg, "")
            }))
            .with_child(
                SubmitButton::new()
                    .class("pwt-scheme-primary")
                    .text("Login")
                    .on_submit(ctx.link().callback(move |_| Msg::Submit))
            );

        Form::new()
            .form_context(self.form_ctx.clone())
            .with_child(panel)
            .into()
    }

}

impl Component for PvePageLogin {
    type Message = Msg;
    type Properties = PageLogin;

    fn create(ctx: &Context<Self>) -> Self {
        let form_ctx = FormContext::new()
            .on_change(ctx.link().callback(|_| Msg::FormDataChange));

        Self {
            form_ctx,
            loading: false,
            login_error: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FormDataChange => {
                self.login_error = None;
                true
            }
            Msg::Submit => {
                self.loading = true;

                //let data = self.form_state.get_submit_data();
                //log::info!("Submit Data {:?}", data);

                let props = ctx.props().clone();
                let link = ctx.link().clone();

                let username = self.form_ctx.read().get_field_text("username");
                let password = self.form_ctx.read().get_field_text("password");
                let realm = self.form_ctx.read().get_field_text("realm");

                //log::info!("Submit {} {}", username, realm);
                wasm_bindgen_futures::spawn_local(async move {
                    match crate::http_login(username, password, realm).await {
                        Ok(info) => {
                            if let Some(on_login) = &props.on_login {
                                on_login.emit(info);
                            }
                            link.send_message(Msg::Login);
                        }
                        Err(err) => {
                            log::error!("ERROR: {:?}", err);
                            link.send_message(Msg::LoginError(err.to_string()));
                        }
                    }
                 });

                true
            }
            Msg::Login => {
                self.loading = false;
                true
            }
            Msg::LoginError(msg) => {
                self.loading = false;
                self.login_error = Some(msg);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = Column::new()
            .class("pwt-fit")
            .class("pwt-align-items-center pwt-justify-content-center")
            .with_child(self.login_form(ctx));

        Column::new()
            .class("pwt-viewport")
            .with_child(TopNavBar::new())
            .with_child(
                Mask::new()
                    .visible(self.loading)
                    .with_child(content)
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