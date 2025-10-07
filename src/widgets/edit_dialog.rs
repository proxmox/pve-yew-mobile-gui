use std::rc::Rc;

use anyhow::Error;
use pwt::state::PersistentState;
use pwt::touch::SideDialog;
use serde_json::Value;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{Key, VComp, VNode};

use proxmox_client::ApiResponseData;

use pwt::impl_yew_std_props_builder;
use pwt::props::{IntoSubmitCallback, SubmitCallback, WidgetStyleBuilder};
use pwt::widget::form::{Checkbox, Form, FormContext, Hidden, ResetButton, SubmitButton};
use pwt::widget::{AlertDialog, Column, Container, Progress, Row};
use pwt::{prelude::*, AsyncPool};

use pwt_macros::builder;

use proxmox_yew_comp::{ApiLoadCallback, IntoApiLoadCallback};

use crate::widgets::editable_property::{PropertyEditorState, RenderPropertyInputPanelFn};

// Like proxmox_yew_comp::EditWindow, but for mobile
//
// - no advanced_checkbox (should be set inside Appbar menu)
// - no "draggable" property
// - no "resizable" property
// - no "autocenter" property

#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct EditDialog {
    /// Yew component key
    #[prop_or_default]
    pub key: Option<Key>,

    /// Window title
    #[prop_or_default]
    pub title: AttrValue,

    /// Show advanced checkbox
    #[prop_or_default]
    #[builder]
    pub advanced_checkbox: bool,

    // Form renderer.
    #[prop_or_default]
    pub renderer: Option<RenderPropertyInputPanelFn>,

    /// Form data loader.
    #[builder_cb(IntoApiLoadCallback, into_api_load_callback, Value)]
    #[prop_or_default]
    pub loader: Option<ApiLoadCallback<Value>>,

    /// Load hook.
    ///
    /// This callback can be used to modify the data returned by the [Self::loader].
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub load_hook: Option<Callback<Value, Result<Value, Error>>>,

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
    ///
    /// On submit, this is called with the data from the [FormContext].
    ///
    /// The [Self::submit_hook] can be used to extract and modify the data
    /// which gets submitted.
    #[builder_cb(IntoSubmitCallback, into_submit_callback, Value)]
    #[prop_or_default]
    pub on_submit: Option<SubmitCallback<Value>>,

    /// Submit hook.
    ///
    /// This callback can used to extract and modify data before
    /// calling the [Self::on_submit] callback.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub submit_hook: Option<Callback<PropertyEditorState, Result<Value, Error>>>,

    /// Reset button press callback.
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    pub on_reset: Option<Callback<()>>,

    /// Data change callback.
    #[builder_cb(IntoEventCallback, into_event_callback, PropertyEditorState)]
    #[prop_or_default]
    pub on_change: Option<Callback<PropertyEditorState>>,

    /// Determines if the window is in edit mode (enabled reset button + dirty tracking)
    ///
    /// Set automatically if a loader is present, can be turned off or on manually with this option.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub edit: Option<bool>,
}

impl EditDialog {
    impl_yew_std_props_builder!();
}

impl EditDialog {
    pub fn new(title: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            title: title.into(),
        })
    }

    pub fn renderer(mut self, renderer: impl Into<RenderPropertyInputPanelFn>) -> Self {
        self.set_renderer(renderer);
        self
    }

    pub fn set_renderer(&mut self, renderer: impl Into<RenderPropertyInputPanelFn>) {
        self.renderer = Some(RenderPropertyInputPanelFn::new(renderer));
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
    ShowAdvanced(bool),
}

#[doc(hidden)]
pub struct PwtEditDialog {
    loading: bool,
    form_ctx: FormContext,
    submit_error: Option<String>,
    load_data: Rc<Value>,
    load_error: Option<String>,
    async_pool: AsyncPool,
    show_advanced: PersistentState<bool>,
}

impl Component for PwtEditDialog {
    type Message = Msg;
    type Properties = EditDialog;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);

        let form_ctx = FormContext::new().on_change(ctx.link().callback(|_| Msg::FormDataChange));

        let show_advanced = PersistentState::new("proxmox-form-show-advanced");
        form_ctx.set_show_advanced(*show_advanced);

        Self {
            form_ctx,
            loading: false,
            submit_error: None,
            load_error: None,
            load_data: Rc::new(Value::Null),
            async_pool: AsyncPool::new(),
            show_advanced,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ShowAdvanced(show_advanced) => {
                self.form_ctx.set_show_advanced(show_advanced);
                self.show_advanced.update(show_advanced);
                true
            }
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
                            if let Some(Value::String(digest)) = api_resp.attribs.get("digest") {
                                value["digest"] = digest.clone().into();
                            }
                        }
                        if let Some(load_hook) = &props.load_hook {
                            match load_hook.emit(value) {
                                Ok(value) => {
                                    self.load_data = Rc::new(value.clone());
                                    self.form_ctx.load_form(value);
                                }
                                Err(err) => self.load_error = Some(err.to_string()),
                            }
                        } else {
                            self.load_data = Rc::new(value.clone());
                            self.form_ctx.load_form(value);
                        }
                    }
                }
                true
            }
            Msg::FormDataChange => {
                if self.submit_error.is_some() {
                    self.submit_error = None;
                }
                if let Some(on_change) = &props.on_change {
                    let state = PropertyEditorState {
                        form_ctx: self.form_ctx.clone(),
                        record: self.load_data.clone(),
                    };
                    on_change.emit(state);
                }
                // Note: we redraw on any data change
                true
            }
            Msg::Submit => {
                if let Some(on_submit) = props.on_submit.clone() {
                    let link = ctx.link().clone();
                    let form_ctx = self.form_ctx.clone();
                    let submit_hook = props.submit_hook.clone();
                    self.loading = true;
                    let state = PropertyEditorState {
                        form_ctx: self.form_ctx.clone(),
                        record: self.load_data.clone(),
                    };
                    self.async_pool.spawn(async move {
                        let result = if let Some(submit_hook) = &submit_hook {
                            submit_hook.emit(state.clone())
                        } else {
                            Ok(form_ctx.get_submit_data())
                        };
                        let result = match result {
                            Ok(submit_data) => on_submit.apply(submit_data).await,
                            Err(err) => Err(err),
                        };
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
            Some(renderer) => {
                let state = PropertyEditorState {
                    form_ctx: self.form_ctx.clone(),
                    record: self.load_data.clone(),
                };
                renderer.apply(state)
            }
            None => html! {},
        };

        let title = Container::new()
            .class("pwt-font-size-title-large")
            .with_child(props.title.clone());

        let input_panel = Column::new()
            .gap(1)
            .class(pwt::css::FlexFit)
            .class(pwt::css::AlignItems::Stretch)
            .class("pwt-font-size-title-medium")
            .with_child(title)
            .with_flex_spacer()
            .with_child(
                Container::new()
                    .class(pwt::css::FlexFit)
                    // This may have scrollable elements, so we Diasble the SideDialog gesture detecture..
                    .onpointerdown(|event: PointerEvent| {
                        event.stop_propagation();
                    })
                    .ontouchstart(|event: TouchEvent| {
                        event.stop_propagation();
                    })
                    .with_child(content),
            );

        let mut toolbar = Row::new().gap(2);

        if props.advanced_checkbox {
            let advanced = Checkbox::new()
                .checked(*self.show_advanced)
                .on_change(ctx.link().callback(Msg::ShowAdvanced))
                .box_label(tr!("Advanced"));

            toolbar.add_child(advanced);
        }

        toolbar.add_flex_spacer();

        if edit_mode {
            toolbar.add_child(
                ResetButton::new()
                    .class("pwt-button-text")
                    .on_reset(props.on_reset.clone()),
            );
        }

        if props.submit_digest && props.loader.is_some() {
            toolbar.add_child(Hidden::new().name("digest").submit_empty(false));
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
            .class(pwt::css::FlexFit)
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
                .style("max-height", "90dvh")
                .with_child(form)
                .with_optional_child(submit_alert)
                .location(pwt::touch::SideDialogLocation::Bottom)
                .on_close(on_close)
                .into(),
        }
    }
}

impl From<EditDialog> for VNode {
    fn from(props: EditDialog) -> Self {
        let key = props.key.clone();
        let comp = VComp::new::<PwtEditDialog>(Rc::new(props), key);
        VNode::from(comp)
    }
}
