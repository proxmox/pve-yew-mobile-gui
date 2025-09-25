use std::rc::Rc;

use gloo_timers::callback::Timeout;
use serde_json::Value;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::props::{IntoSubmitCallback, SubmitCallback};
use pwt::widget::{Column, Container, List, ListTile};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::utils::render_boolean;
use proxmox_yew_comp::{ApiLoadCallback, IntoApiLoadCallback};

use pwt_macros::builder;

use crate::widgets::{form_list_tile, EditDialog, EditableProperty};

#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct PropertyList {
    /// List of property definitions
    pub properties: Rc<Vec<EditableProperty>>,

    /// Data loader.
    #[builder_cb(IntoApiLoadCallback, into_api_load_callback, Value)]
    #[prop_or_default]
    pub loader: Option<ApiLoadCallback<Value>>,

    /// Submit callback.
    #[builder_cb(IntoSubmitCallback, into_submit_callback, Value)]
    #[prop_or_default]
    pub on_submit: Option<SubmitCallback<Value>>,
}

impl PropertyList {
    pub fn new(properties: Rc<Vec<EditableProperty>>) -> Self {
        yew::props!(Self { properties })
    }
}

pub enum Msg {
    Load,
    LoadResult(Result<Value, String>),
    ShowDialog(Option<Html>),
    EditProperty(EditableProperty),
}

pub struct PvePropertyList {
    data: Option<Result<Value, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    edit_dialog: Option<Html>,
}

impl PvePropertyList {
    fn property_tile(
        &self,
        ctx: &Context<Self>,
        record: &Value,
        property: &EditableProperty,
    ) -> ListTile {
        let name = &property.name.as_str();
        let value = record.get(name);

        let value_text: Html = match (value, &property.renderer) {
            (None | Some(Value::Null), _) => {
                let placeholder = if let Some(placeholder) = &property.placeholder {
                    placeholder.to_string().into()
                } else {
                    String::from("-")
                };
                Container::new()
                    .class(pwt::css::Opacity::Half)
                    .with_child(placeholder)
                    .into()
            }

            (Some(value), None) => match value {
                Value::String(value) => value.clone(),
                Value::Bool(value) => render_boolean(*value),
                Value::Number(n) => n.to_string(),
                v => v.to_string(),
            }
            .into(),
            (Some(value), Some(renderer)) => renderer.apply(&*property.name, &value, &record),
        };

        let list_tile = if property.single_row {
            let trailing: Html = Container::new()
                .style("text-align", "end")
                .with_child(value_text)
                .into();
            form_list_tile(property.title.clone(), (), trailing)
        } else {
            form_list_tile(property.title.clone(), value_text, ())
        };

        if property.render_input_panel.is_some() {
            list_tile
                .interactive(true)
                .on_activate(ctx.link().callback({
                    let property = property.clone();
                    move |_| Msg::EditProperty(property.clone())
                }))
        } else {
            list_tile
        }
    }

    fn view_property(&self, ctx: &Context<Self>, record: &Value) -> Html {
        let props = ctx.props();

        let mut tiles: Vec<ListTile> = Vec::new();

        for item in props.properties.iter() {
            let value = record.get(&*item.name);
            if !item.required && (value.is_none() || value == Some(&Value::Null)) {
                continue;
            }

            let mut list_tile = self.property_tile(ctx, record, item);
            list_tile.set_key(item.name.clone());

            tiles.push(list_tile);
        }

        Column::new()
            .class(pwt::css::FlexFit)
            .with_child(
                List::from_tiles(tiles)
                    .virtual_scroll(Some(false))
                    //fixme: .separator(props.separator)
                    .grid_template_columns("1fr auto")
                    .class(pwt::css::FlexFit),
            )
            .with_optional_child(self.edit_dialog.clone())
            .into()
    }
}

impl Component for PvePropertyList {
    type Message = Msg;
    type Properties = PropertyList;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
            reload_timeout: None,
            load_guard: None,
            edit_dialog: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::EditProperty(property) => {
                let dialog = EditDialog::from(property.clone())
                    .on_done(ctx.link().callback(|_| Msg::ShowDialog(None)))
                    .loader(props.loader.clone())
                    .on_submit(props.on_submit.clone())
                    .into();
                self.edit_dialog = Some(dialog);
            }
            Msg::Load => {
                self.reload_timeout = None;
                let link = ctx.link().clone();
                if let Some(loader) = props.loader.clone() {
                    self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                        let result = loader.apply().await;
                        let data = match result {
                            Ok(result) => Ok(result.data),
                            Err(err) => Err(err.to_string()),
                        };
                        link.send_message(Msg::LoadResult(data));
                    }));
                }
            }
            Msg::LoadResult(result) => {
                self.data = Some(result);
                let link = ctx.link().clone();
                self.reload_timeout = Some(Timeout::new(3000, move || {
                    link.send_message(Msg::Load);
                }));
            }
            Msg::ShowDialog(dialog) => {
                if dialog.is_none() && self.reload_timeout.is_some() {
                    ctx.link().send_message(Msg::Load);
                }
                self.edit_dialog = dialog;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        crate::widgets::render_loaded_data(&self.data, |data| self.view_property(ctx, data))
    }
}

impl From<PropertyList> for VNode {
    fn from(props: PropertyList) -> Self {
        let comp = VComp::new::<PvePropertyList>(Rc::new(props), None);
        VNode::from(comp)
    }
}
