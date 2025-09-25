use std::rc::Rc;

use anyhow::Error;
use derivative::Derivative;
use proxmox_yew_comp::utils::render_boolean;
use serde_json::{Number, Value};

use pwt::prelude::*;
use pwt::widget::form::{Checkbox, Field, FormContext};
use pwt::widget::Row;

use pwt_macros::builder;
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::widgets::EditDialog;

/// For use with [EditableProperty]
#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct RenderPropertyFn(
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    #[allow(clippy::type_complexity)]
    Rc<dyn Fn(&str, &Value, &Value) -> Html>,
);

impl RenderPropertyFn {
    /// Creates a new [`RenderKVGridRecordFn`]
    pub fn new(renderer: impl 'static + Fn(&str, &Value, &Value) -> Html) -> Self {
        Self(Rc::new(renderer))
    }

    /// Apply the render function
    pub fn apply(&self, name: &str, value: &Value, record: &Value) -> Html {
        (self.0)(name, value, record)
    }
}

/// For use with [EditableProperty]
#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct RenderPropertyInputPanelFn(
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    #[allow(clippy::type_complexity)]
    Rc<dyn Fn(FormContext, Rc<Value>) -> Html>,
);

impl RenderPropertyInputPanelFn {
    /// Creates a new instance.
    pub fn new(renderer: impl Into<RenderPropertyInputPanelFn>) -> Self {
        renderer.into()
    }

    /// Apply the render function
    pub fn apply(&self, form_ctx: FormContext, record: Rc<Value>) -> Html {
        (self.0)(form_ctx, record)
    }
}

impl<F: 'static + Fn(FormContext, Rc<Value>) -> Html> From<F> for RenderPropertyInputPanelFn {
    fn from(renderer: F) -> Self {
        Self(Rc::new(renderer))
    }
}

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

    /// Show advanced checkbox
    #[builder]
    pub advanced_checkbox: bool,

    #[builder(IntoPropValue, into_prop_value)]
    pub placeholder: Option<AttrValue>,
    pub renderer: Option<RenderPropertyFn>,

    /// Load hook.
    #[builder(IntoPropValue, into_prop_value)]
    pub load_hook: Option<Callback<Value, Result<Value, Error>>>,

    /// Submit hook.
    #[builder(IntoPropValue, into_prop_value)]
    pub submit_hook: Option<Callback<FormContext, Result<Value, Error>>>,

    /// Data change callback.
    #[builder_cb(IntoEventCallback, into_event_callback, FormContext)]
    pub on_change: Option<Callback<FormContext>>,

    /// Edit input panel builder
    pub render_input_panel: Option<RenderPropertyInputPanelFn>,
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
            //loader: None,
            load_hook: None,
            submit_hook: None,
            //on_submit: None,
            on_change: None,
            render_input_panel: None,
            advanced_checkbox: false,
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
                let text: String = match value {
                    Value::Bool(value) => render_boolean(*value),
                    Value::Number(n) if n.as_u64() == Some(0) || n.as_u64() == Some(1) => {
                        render_boolean(n.as_u64() == Some(1))
                    }
                    Value::Null => match default {
                        Some(default) => render_boolean(default),
                        None => "-".into(),
                    },
                    _ => value.to_string(),
                };
                text.into()
            })
            .render_input_panel(move |_form_ctx, _record| {
                Row::new()
                    .with_flex_spacer()
                    .with_child(Checkbox::new().name(name.to_string()).switch(true))
                    .into()
            })
    }

    pub fn new_string(name: impl Into<AttrValue>, title: impl Into<AttrValue>) -> Self {
        let name = name.into();
        Self::new(name.clone(), title).render_input_panel(move |_form_ctx, _record| {
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
        self.renderer = Some(RenderPropertyFn::new(renderer));
    }

    pub fn render_input_panel(mut self, renderer: impl Into<RenderPropertyInputPanelFn>) -> Self {
        self.set_render_input_panel(renderer);
        self
    }

    pub fn set_render_input_panel(&mut self, renderer: impl Into<RenderPropertyInputPanelFn>) {
        self.render_input_panel = Some(RenderPropertyInputPanelFn::new(renderer));
    }
}

impl From<EditableProperty> for EditDialog {
    fn from(property: EditableProperty) -> Self {
        let renderer = match property.render_input_panel {
            Some(renderer) => renderer,
            None => RenderPropertyInputPanelFn::new(|_, _| html! {}),
        };

        EditDialog::new(property.title)
            .advanced_checkbox(property.advanced_checkbox)
            .submit_hook(property.submit_hook)
            .load_hook(property.load_hook)
            .on_change(property.on_change)
            .renderer(renderer)
    }
}
