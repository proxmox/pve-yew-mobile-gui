use std::collections::HashSet;
use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use serde_json::{Map, Value};

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::props::{IntoSubmitCallback, SubmitCallback};
use pwt::widget::{Column, Container, List, ListTile};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{ApiLoadCallback, IntoApiLoadCallback};

use pwt_macros::builder;

use crate::widgets::{form_list_tile, EditDialog, EditableProperty};
use crate::QemuPendingConfigValue;

#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct PendingPropertyList {
    /// List of property definitions
    pub properties: Rc<Vec<EditableProperty>>,

    /// Load property list with pending changes information.
    #[builder_cb(IntoApiLoadCallback, into_api_load_callback, Vec<QemuPendingConfigValue>)]
    #[prop_or_default]
    pub pending_loader: Option<ApiLoadCallback<Vec<QemuPendingConfigValue>>>,

    /// Loader passed to the EditDialog
    #[builder_cb(IntoApiLoadCallback, into_api_load_callback, Value)]
    #[prop_or_default]
    pub editor_loader: Option<ApiLoadCallback<Value>>,

    /// Submit callback.
    #[builder_cb(IntoSubmitCallback, into_submit_callback, Value)]
    #[prop_or_default]
    pub on_submit: Option<SubmitCallback<Value>>,
}

impl PendingPropertyList {
    pub fn new(properties: Rc<Vec<EditableProperty>>) -> Self {
        yew::props!(Self { properties })
    }
}

/// Parse PVE pending configuration array
///
/// Returns 2 Objects, containing current and pending configuration,
/// and the set of deleted keys.
pub fn pve_pending_config_array_to_objects(
    data: Vec<QemuPendingConfigValue>,
) -> Result<(Value, Value, HashSet<String>), Error> {
    let mut current = Map::new();
    let mut pending = Map::new();
    let mut changes = HashSet::new();

    for item in data.iter() {
        if let Some(value) = item.value.clone() {
            current.insert(item.key.clone(), value);
        }
        if matches!(item.delete, Some(1) | Some(2)) {
            changes.insert(item.key.clone());
            continue;
        }
        if let Some(value) = item.pending.clone() {
            changes.insert(item.key.clone());
            pending.insert(item.key.clone(), value);
        } else if let Some(value) = item.value.clone() {
            pending.insert(item.key.clone(), value);
        }
    }

    Ok((Value::Object(current), Value::Object(pending), changes))
}

pub enum Msg {
    Load,
    LoadResult(Result<Vec<QemuPendingConfigValue>, String>),
    ShowDialog(Option<Html>),
    EditProperty(EditableProperty),
}

pub struct PvePendingPropertyList {
    data: Option<Result<(Value, Value, HashSet<String>), String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    edit_dialog: Option<Html>,
}

impl PvePendingPropertyList {
    fn property_tile(
        &self,
        ctx: &Context<Self>,
        current: &Value,
        pending: &Value,
        property: &EditableProperty,
    ) -> ListTile {
        let name = &property.name.as_str();

        let render_value = |data_record: &Value| {
            let value = &data_record[name];
            match value {
                Value::Null => property
                    .placeholder
                    .clone()
                    .unwrap_or(AttrValue::Static("-"))
                    .to_string()
                    .into(),
                other => property
                    .renderer
                    .clone()
                    .unwrap()
                    .apply(name, other, &data_record),
            }
        };

        let mut value = render_value(current);
        let new_value = render_value(pending);

        if value != new_value {
            value = html! {<><div>{value}</div><div style="line-height: 1.4em;" class="pwt-color-warning">{new_value}</div></>};
        }

        // fixme: revert button?

        let list_tile = if property.single_row {
            // fixme: single row pending values???
            let trailing: Html = Container::new()
                .style("text-align", "end")
                .with_child(value)
                .into();
            form_list_tile(property.title.clone(), (), trailing)
        } else {
            form_list_tile(property.title.clone(), value, ())
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

    fn view_property(
        &self,
        ctx: &Context<Self>,
        (record, pending, _changes): &(Value, Value, HashSet<String>),
    ) -> Html {
        let props = ctx.props();

        let mut tiles: Vec<ListTile> = Vec::new();

        for item in props.properties.iter() {
            let value = record.get(&*item.name);
            if !item.required && (value.is_none() || value == Some(&Value::Null)) {
                continue;
            }

            let mut tile = self.property_tile(ctx, record, pending, item);
            tile.set_key(item.name.clone());

            tiles.push(tile);
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

impl Component for PvePendingPropertyList {
    type Message = Msg;
    type Properties = PendingPropertyList;

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
                    .loader(props.editor_loader.clone())
                    .on_submit(props.on_submit.clone())
                    .into();
                self.edit_dialog = Some(dialog);
            }
            Msg::Load => {
                self.reload_timeout = None;
                let link = ctx.link().clone();
                if let Some(loader) = props.pending_loader.clone() {
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
                self.data = match result {
                    Ok(data) => Some(
                        pve_pending_config_array_to_objects(data).map_err(|err| err.to_string()),
                    ),
                    Err(err) => Some(Err(err.to_string())),
                };
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

impl From<PendingPropertyList> for VNode {
    fn from(props: PendingPropertyList) -> Self {
        let comp = VComp::new::<PvePendingPropertyList>(Rc::new(props), None);
        VNode::from(comp)
    }
}
