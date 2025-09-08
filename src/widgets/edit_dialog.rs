use std::marker::PhantomData;
use std::rc::Rc;

use anyhow::Error;
use derivative::Derivative;
use pwt::touch::SideDialog;
use serde::Serialize;
use serde_json::Value;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{Key, VComp, VNode};

use proxmox_client::ApiResponseData;

use pwt::impl_yew_std_props_builder;
use pwt::props::{
    IntoOptionalRenderFn, IntoSubmitCallback, RenderFn, SubmitCallback, WidgetStyleBuilder,
};
use pwt::widget::form::{Form, FormContext, Hidden, ResetButton, SubmitButton};
use pwt::widget::{AlertDialog, Column, Progress, Row};
use pwt::{prelude::*, AsyncPool};

use pwt_macros::builder;

use proxmox_yew_comp::{ApiLoadCallback, IntoApiLoadCallback};

// Like proxmox_yew_comp::EditWindow, but for mobile
//
// - no advanced_checkbox (should be set inside Appbar menu)
// - no "draggable" property
// - no "resizable" property
// - no "autocenter" property

#[derive(Properties, Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
#[builder]
pub struct EditDialog<T: Serialize> {
    /// Yew component key
    #[prop_or_default]
    pub key: Option<Key>,

    /// Window title
    #[prop_or_default]
    pub title: AttrValue,

    // Form renderer.
    #[prop_or_default]
    pub renderer: Option<RenderFn<(FormContext, Option<Value>)>>,

    /// Form data loader.
    #[builder_cb(IntoApiLoadCallback, into_api_load_callback, T)]
    #[prop_or_default]
    pub loader: Option<ApiLoadCallback<T>>,

    /// Submit button text.
    ///
    /// Default is Add, or Update if there is a loader.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub submit_text: Option<AttrValue>,

    /// Submit the digest if the loader returned one.
    #[prop_or(true)]
    #[builder]
    pub submit_digest: bool,

    /// Close/Abort callback.
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub on_close: Option<Callback<()>>,

    /// Done callback, called after Close, Abort or Submit.
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub on_done: Option<Callback<()>>,

    /// Submit callback.
    #[prop_or_default]
    pub on_submit: Option<SubmitCallback<FormContext>>,

    /// Reset button press callback.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_reset: Option<Callback<()>>,

    /// Data change callback.
    #[builder_cb(IntoEventCallback, into_event_callback, FormContext)]
    #[prop_or_default]
    pub on_change: Option<Callback<FormContext>>,

    /// Determines if the window is in edit mode (enabled reset button + dirty tracking)
    ///
    /// Set automatically if a loader is present, can be turned off or on manually with this option.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub edit: Option<bool>,
}

impl<T: Serialize> EditDialog<T> {
    impl_yew_std_props_builder!();
}

impl<T: Serialize> EditDialog<T> {
    pub fn new(title: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            title: title.into(),
        })
    }

    pub fn renderer(
        mut self,
        renderer: impl IntoOptionalRenderFn<(FormContext, Option<Value>)>,
    ) -> Self {
        self.set_renderer(renderer);
        self
    }

    pub fn set_renderer(
        &mut self,
        renderer: impl IntoOptionalRenderFn<(FormContext, Option<Value>)>,
    ) {
        self.renderer = renderer.into_optional_render_fn();
    }

    pub fn on_submit(mut self, callback: impl IntoSubmitCallback<FormContext>) -> Self {
        self.on_submit = callback.into_submit_callback();
        self
    }

    pub fn is_edit(&self) -> bool {
        if let Some(is_edit) = self.edit {
            is_edit
        } else {
            self.loader.is_some()
        }
    }
}

pub enum Msg {
    FormDataChange,
    Submit,
    SubmitResult(Result<(), Error>),
    ClearSubmitError,
    Load,
    LoadResult(Result<ApiResponseData<Value>, Error>),
}

#[doc(hidden)]
pub struct PwtEditDialog<T> {
    loading: bool,
    form_ctx: FormContext,
    submit_error: Option<String>,
    load_data: Option<Value>,
    load_error: Option<String>,
    async_pool: AsyncPool,
    _phantom: PhantomData<T>,
}

impl<T: 'static + Serialize> Component for PwtEditDialog<T> {
    type Message = Msg;
    type Properties = EditDialog<T>;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);

        let form_ctx = FormContext::new().on_change(ctx.link().callback(|_| Msg::FormDataChange));

        Self {
            form_ctx,
            loading: false,
            submit_error: None,
            load_error: None,
            load_data: None,
            async_pool: AsyncPool::new(),
            _phantom: PhantomData,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ClearSubmitError => {
                self.submit_error = None;
                true
            }
            Msg::Load => {
                if let Some(loader) = props.loader.clone() {
                    self.loading = true;
                    let link = ctx.link().clone();
                    self.async_pool.spawn(async move {
                        let res = loader.apply().await;
                        link.send_message(Msg::LoadResult(res.map(|r| ApiResponseData {
                            data: serde_json::to_value(r.data).unwrap(),
                            attribs: r.attribs,
                        })));
                    });
                }
                true
            }
            Msg::LoadResult(result) => {
                self.loading = false;
                match result {
                    Err(err) => self.load_error = Some(err.to_string()),
                    Ok(api_resp) => {
                        let mut value = api_resp.data;
                        if props.submit_digest {
                            if let Some(digest) = api_resp.attribs.get("digest") {
                                value["digest"] = digest.clone();
                            }
                        }
                        self.load_data = Some(value.clone());
                        self.form_ctx.load_form(value);
                    }
                }
                true
            }
            Msg::FormDataChange => {
                if self.submit_error.is_some() {
                    self.submit_error = None;
                }
                if let Some(on_change) = &props.on_change {
                    on_change.emit(self.form_ctx.clone());
                }
                // Note: we redraw on any data change
                true
            }
            Msg::Submit => {
                if let Some(on_submit) = props.on_submit.clone() {
                    let link = ctx.link().clone();
                    let form_ctx = self.form_ctx.clone();
                    self.loading = true;
                    self.async_pool.spawn(async move {
                        let result = on_submit.apply(form_ctx).await;
                        link.send_message(Msg::SubmitResult(result));
                    });
                }
                true
            }
            Msg::SubmitResult(result) => {
                self.loading = false;
                match result {
                    Ok(_) => {
                        self.submit_error = None;
                        if let Some(on_done) = &props.on_done {
                            on_done.emit(());
                        }
                    }
                    Err(err) => {
                        self.submit_error = Some(err.to_string());
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let link = ctx.link();
        let form_ctx = self.form_ctx.clone();

        let edit_mode = props.is_edit();
        let renderer = props.renderer.clone();
        let loading = self.loading;

        let content = match &renderer {
            Some(renderer) => renderer.apply(&(form_ctx.clone(), self.load_data.clone())),
            None => html! {},
        };

        let input_panel = Column::new()
            .gap(1)
            .class(pwt::css::Flex::Fill)
            .class(pwt::css::AlignItems::Stretch)
            .class("pwt-font-size-title-medium")
            .with_child(props.title.clone())
            .with_flex_spacer()
            .with_child(content);

        let mut toolbar = Row::new().gap(2).with_flex_spacer();

        if edit_mode {
            toolbar.add_child(
                ResetButton::new()
                    .class("pwt-button-text")
                    .on_reset(props.on_reset.clone()),
            );

            if props.submit_digest {
                toolbar.add_child(Hidden::new().name("digest").submit_empty(false));
            }
        }

        let submit_text = match &props.submit_text {
            Some(submit_text) => submit_text.to_string(),
            None => {
                if edit_mode {
                    tr!("Update")
                } else {
                    tr!("Add")
                }
            }
        };

        toolbar.add_child(
            SubmitButton::new()
                .class("pwt-scheme-primary")
                .text(submit_text)
                .check_dirty(edit_mode)
                .on_submit(link.callback(|_| Msg::Submit)),
        );

        let form = Form::new()
            .form_context(form_ctx.clone())
            .class(pwt::css::FlexFit)
            .with_child(
                Column::new()
                    .class(pwt::css::FlexFit)
                    .padding(2)
                    .gap(4)
                    .with_child(input_panel)
                    .with_child(toolbar),
            );

        let form = Column::new()
            .class("pwt-flex-fit")
            .style("position", "relative")
            .with_child(
                Progress::new()
                    .class("pwt-delay-visibility")
                    .style("position", "absolute")
                    .style("left", "0")
                    .style("right", "0")
                    .style("visibility", (!loading).then(|| "hidden")),
            )
            .with_child(form.style("visibility", loading.then(|| "hidden")));

        let on_close = {
            let on_close = props.on_close.clone();
            let on_done = props.on_done.clone();

            if on_close.is_some() || on_done.is_some() {
                Some(move |()| {
                    if let Some(on_close) = &on_close {
                        on_close.emit(());
                    }
                    if let Some(on_done) = &on_done {
                        on_done.emit(());
                    }
                })
            } else {
                None
            }
        };

        let submit_alert = self.submit_error.as_ref().map({
            let link = ctx.link();
            move |msg| AlertDialog::new(msg).on_close(link.callback(|_| Msg::ClearSubmitError))
        });

        match &self.load_error {
            Some(msg) => AlertDialog::new(msg).on_close(on_close).into(),
            None => SideDialog::new()
                .with_child(form)
                .with_optional_child(submit_alert)
                .location(pwt::touch::SideDialogLocation::Bottom)
                .on_close(on_close)
                .into(),
        }
    }
}

impl<T: 'static + Serialize> From<EditDialog<T>> for VNode {
    fn from(props: EditDialog<T>) -> Self {
        let key = props.key.clone();
        let comp = VComp::new::<PwtEditDialog<T>>(Rc::new(props), key);
        VNode::from(comp)
    }
}
