use proxmox_yew_comp::utils::render_boolean;
use serde_json::Value;

use proxmox_yew_comp::{ApiLoadCallback, IntoApiLoadCallback, RenderKVGridRecordFn};
use pwt::prelude::*;
use pwt::props::{IntoOptionalRenderFn, IntoSubmitCallback, RenderFn, SubmitCallback};
use pwt::widget::form::{Checkbox, Field, FormContext};
use pwt::widget::Row;

use pwt_macros::builder;
use yew::html::IntoPropValue;

#[derive(Clone, PartialEq)]
#[builder]
pub struct EditableProperty {
    pub name: AttrValue,
    pub title: AttrValue,
    #[builder]
    pub required: bool,
    /// Use single line layout (title/value in one row).
    ///
    /// Only used for the mobile list layout.
    #[builder]
    pub single_row: bool,

    #[builder(IntoPropValue, into_prop_value)]
    pub placeholder: Option<AttrValue>,
    pub renderer: Option<RenderKVGridRecordFn>,
    /// Submit callback.
    pub on_submit: Option<SubmitCallback<FormContext>>,

    /// Data loader.
    #[builder_cb(IntoApiLoadCallback, into_api_load_callback, Value)]
    pub loader: Option<ApiLoadCallback<Value>>,

    /// Edit input panel builder
    pub render_input_panel: Option<RenderFn<FormContext>>,
}

impl EditableProperty {
    pub fn new(name: impl Into<AttrValue>, title: impl Into<AttrValue>) -> Self {
        Self {
            name: name.into(),
            title: title.into(),
            required: false,
            single_row: false,
            placeholder: None,
            renderer: None,
            loader: None,
            on_submit: None,
            render_input_panel: None,
        }
    }

    pub fn new_bool(
        name: impl Into<AttrValue>,
        title: impl Into<AttrValue>,
        default: impl IntoPropValue<Option<bool>>,
    ) -> Self {
        let name = name.into();
        let default = default.into_prop_value();
        Self::new(name.clone(), title)
            .single_row(true)
            .placeholder(default.map(|default| render_boolean(default)))
            .renderer(move |_name, value, _data| {
                let text: String = match value.as_bool() {
                    Some(value) => render_boolean(value),
                    None => match default {
                        Some(default) => render_boolean(default),
                        None => "-".into(),
                    },
                };
                text.into()
            })
            .render_input_panel(move |_form_ctx: &FormContext| {
                Row::new()
                    .with_flex_spacer()
                    .with_child(Checkbox::new().name(name.to_string()).switch(true))
                    .into()
            })
    }

    pub fn new_string(name: impl Into<AttrValue>, title: impl Into<AttrValue>) -> Self {
        let name = name.into();
        Self::new(name.clone(), title).render_input_panel(move |_form_ctx: &FormContext| {
            Field::new()
                .name(name.to_string())
                .submit_empty(true)
                .into()
        })
    }

    pub fn renderer(mut self, renderer: impl 'static + Fn(&str, &Value, &Value) -> Html) -> Self {
        self.set_renderer(renderer);
        self
    }

    pub fn set_renderer(&mut self, renderer: impl 'static + Fn(&str, &Value, &Value) -> Html) {
        self.renderer = Some(RenderKVGridRecordFn::new(renderer));
    }

    pub fn render_input_panel(mut self, renderer: impl IntoOptionalRenderFn<FormContext>) -> Self {
        self.set_render_input_panel(renderer);
        self
    }

    pub fn set_render_input_panel(&mut self, renderer: impl IntoOptionalRenderFn<FormContext>) {
        self.render_input_panel = renderer.into_optional_render_fn();
    }

    pub fn on_submit(mut self, callback: impl IntoSubmitCallback<FormContext>) -> Self {
        self.on_submit = callback.into_submit_callback();
        self
    }
}
