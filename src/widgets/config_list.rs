use std::rc::Rc;

use gloo_timers::callback::Timeout;
use serde_json::Value;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::props::{IntoSubmitCallback, SubmitCallback};
use pwt::widget::form::FormContext;
use pwt::widget::{Column, Container, List, ListTile};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::utils::render_boolean;
use proxmox_yew_comp::{ApiLoadCallback, IntoApiLoadCallback};

use pwt_macros::builder;

use crate::widgets::{form_list_tile, EditDialog, EditableProperty};

#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct ConfigList {
    /// List of property definitions
    pub properties: Rc<Vec<EditableProperty>>,

    /// Data loader.
    #[builder_cb(IntoApiLoadCallback, into_api_load_callback, Value)]
    #[prop_or_default]
    pub loader: Option<ApiLoadCallback<Value>>,

    /// Default Submit callback (for properties without on_submit)
    #[prop_or_default]
    pub on_submit: Option<SubmitCallback<FormContext>>,
}

impl ConfigList {
    pub fn new(properties: Rc<Vec<EditableProperty>>) -> Self {
        yew::props!(Self { properties })
    }

    pub fn on_submit(mut self, callback: impl IntoSubmitCallback<FormContext>) -> Self {
        self.on_submit = callback.into_submit_callback();
        self
    }
}

pub enum Msg {
    Load,
    LoadResult(Result<Value, String>),
    CloseDialog,
    ShowDialog(Html),
}

pub struct PveConfigList {
    data: Option<Result<Value, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    edit_dialog: Option<Html>,
}

impl PveConfigList {
    fn view_config(&self, ctx: &Context<Self>, record: &Value) -> Html {
        let props = ctx.props();

        let mut tiles: Vec<ListTile> = Vec::new();

        for item in props.properties.iter() {
            let value = record.get(&*item.name);
            if !item.required && (value.is_none() || value == Some(&Value::Null)) {
                continue;
            }

            let value_text: Html = match (value, &item.renderer) {
                (None | Some(Value::Null), _) => {
                    let placeholder = if let Some(placeholder) = &item.placeholder {
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
                (Some(value), Some(renderer)) => renderer.apply(&*item.name, &value, &record),
            };

            let mut list_tile = if item.single_row {
                let trailing: Html = Container::new()
                    .style("text-align", "end")
                    .with_child(value_text)
                    .into();
                form_list_tile(item.title.clone(), (), trailing)
            } else {
                form_list_tile(item.title.clone(), value_text, ())
            };

            let on_submit = match item.on_submit.clone() {
                Some(on_submit) => Some(on_submit),
                None => props.on_submit.clone(),
            };

            if let Some(on_submit) = on_submit {
                if item.render_input_panel.is_some() {
                    list_tile.set_interactive(true);
                    list_tile.add_onclick({
                        let link = ctx.link().clone();
                        let item = item.clone();
                        let loader = match item.loader {
                            Some(loader) => Some(loader),
                            None => props.loader.clone(),
                        };

                        move |_| {
                            if let Some(render_input_panel) = item.render_input_panel.clone() {
                                let dialog = EditDialog::new(item.title.clone())
                                    .on_done(link.callback(|_| Msg::CloseDialog))
                                    .loader(loader.clone())
                                    .on_submit(Some(on_submit.clone()))
                                    .renderer(render_input_panel);

                                link.send_message(Msg::ShowDialog(dialog.into()));
                            }
                        }
                    });
                }
            }

            list_tile.set_key(item.name.clone());

            tiles.push(list_tile);
        }

        Column::new()
            .class(pwt::css::FlexFit)
            .with_child(
                List::new(tiles.len() as u64, move |pos| tiles[pos as usize].clone())
                    .virtual_scroll(Some(false))
                    //fixme: .separator(props.separator)
                    .grid_template_columns("1fr auto")
                    .class(pwt::css::FlexFit),
            )
            .with_optional_child(self.edit_dialog.clone())
            .into()
    }
}

impl Component for PveConfigList {
    type Message = Msg;
    type Properties = ConfigList;

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
                self.edit_dialog = Some(dialog);
            }
            Msg::CloseDialog => {
                if self.reload_timeout.is_some() {
                    ctx.link().send_message(Msg::Load);
                }

                self.edit_dialog = None;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        crate::widgets::render_loaded_data(&self.data, |data| self.view_config(ctx, data))
    }
}

impl From<ConfigList> for VNode {
    fn from(props: ConfigList) -> Self {
        let comp = VComp::new::<PveConfigList>(Rc::new(props), None);
        VNode::from(comp)
    }
}
