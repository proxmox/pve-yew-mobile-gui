use std::collections::HashSet;
use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use proxmox_yew_comp::utils::render_boolean;
use pwt::touch::SnackBar;
use serde_json::{json, Map, Value};

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::props::{IntoOptionalInlineHtml, IntoSubmitCallback, SubmitCallback};
use pwt::touch::SnackBarContextExt;
use pwt::widget::{ActionIcon, Column, Fa, List, ListTile, Row};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{ApiLoadCallback, IntoApiLoadCallback};

use pwt_macros::builder;

use crate::widgets::{title_subtitle_column, EditDialog, EditableProperty};
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
    pub fn render_property_value(
        current: &Value,
        pending: &Value,
        property: &EditableProperty,
    ) -> (Html, Option<Html>) {
        let name = &property.name.as_str();
        let render_value = |data_record: &Value| {
            let value = &data_record[name];
            match (value, &property.renderer) {
                (Value::Null, _) => property
                    .placeholder
                    .clone()
                    .unwrap_or(AttrValue::Static("-"))
                    .to_string()
                    .into(),
                (value, None) => match value {
                    Value::String(value) => value.clone(),
                    Value::Bool(value) => render_boolean(*value),
                    Value::Number(n) => n.to_string(),
                    v => v.to_string(),
                }
                .into(),
                (other, Some(renderer)) => renderer.apply(name, other, &data_record),
            }
        };

        let value = render_value(current);
        let new_value = render_value(pending);

        if value != new_value {
            //value = html! {<><div>{value}</div><div style="line-height: 1.4em;" class="pwt-color-warning">{new_value}</div></>};
            (value, Some(new_value))
        } else {
            (value, None)
        }
    }

    /// Render a ListTile with a single child.
    ///
    /// Suitable for a "grid-template-columns: 1fr".
    pub fn render_list_tile(
        current: &Value,
        pending: &Value,
        property: &EditableProperty,
        trailing: impl IntoOptionalInlineHtml,
        on_revert: Callback<Event>,
    ) -> ListTile {
        Self::render_list_tile_internal(current, pending, property, None, trailing, on_revert)
    }

    /// Render a ListTile with a two children, icon + rest.
    ///
    /// Suitable for a "grid-template-columns: "auto 1fr".
    pub fn render_icon_list_tile(
        current: &Value,
        pending: &Value,
        property: &EditableProperty,
        icon: Fa,
        trailing: impl IntoOptionalInlineHtml,
        on_revert: Callback<Event>,
    ) -> ListTile {
        Self::render_list_tile_internal(current, pending, property, Some(icon), trailing, on_revert)
    }

    // Note: We do not use 3 columns so that we do not waste space on the right side.
    fn render_list_tile_internal(
        current: &Value,
        pending: &Value,
        property: &EditableProperty,
        icon: Option<Fa>,
        trailing: impl IntoOptionalInlineHtml,
        on_revert: Callback<Event>,
    ) -> ListTile {
        let (value, new_value) = Self::render_property_value(current, pending, property);

        let revert: Html = ActionIcon::new("fa fa-undo")
            .on_activate(on_revert.clone())
            .into();

        if let Some(new_value) = new_value {
            let subtitle = html! {<><div>{value}</div><div style="line-height: 1.4em;" class="pwt-color-warning">{new_value}</div></>};
            let content: Html = Row::new()
                .class(pwt::css::AlignItems::Center)
                .class(pwt::css::JustifyContent::End)
                .gap(1)
                .with_child(title_subtitle_column(property.title.clone(), subtitle))
                .with_flex_spacer()
                .with_child(revert)
                .with_optional_child(trailing.into_optional_inline_html())
                .into();

            ListTile::new()
                .class(pwt::css::AlignItems::Center)
                //.class("pwt-column-gap-2")
                .class("pwt-gap-2")
                .border_bottom(true)
                .with_optional_child(icon.map(|i| i.fixed_width().large_2x()))
                .with_child(content)
        } else {
            let trailing = trailing.into_optional_inline_html();

            ListTile::new()
                .class(pwt::css::AlignItems::Center)
                //.class("pwt-column-gap-2")
                .class("pwt-gap-2")
                .border_bottom(true)
                .with_optional_child(icon.map(|i| i.fixed_width().large_2x()))
                .with_child(
                    Row::new()
                        .with_child(title_subtitle_column(property.title.clone(), value))
                        .with_flex_spacer()
                        .with_optional_child(trailing.into_optional_inline_html()),
                )
        }
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
    Revert(EditableProperty),
    RevertResult(Result<(), Error>),
}

pub struct PvePendingPropertyList {
    data: Option<Result<(Value, Value, HashSet<String>), String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    revert_guard: Option<AsyncAbortGuard>,
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
        let on_revert = Callback::from({
            let property = property.clone();
            ctx.link()
                .callback(move |_: Event| Msg::Revert(property.clone()))
        });

        let list_tile =
            PendingPropertyList::render_list_tile(current, pending, property, (), on_revert);

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
                    .grid_template_columns("1fr")
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
            revert_guard: None,
            edit_dialog: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Revert(property) => {
                let link = ctx.link().clone();
                let keys = match property.revert_keys.as_deref() {
                    Some(keys) => keys.iter().map(|a| a.to_string()).collect(),
                    None => vec![property.name.to_string()],
                };
                if let Some(on_submit) = props.on_submit.clone() {
                    let param = json!({ "revert": keys });
                    self.revert_guard = Some(AsyncAbortGuard::spawn(async move {
                        let result = on_submit.apply(param).await;
                        link.send_message(Msg::RevertResult(result));
                    }));
                }
            }
            Msg::RevertResult(result) => {
                if let Err(err) = result {
                    ctx.link().show_snackbar(
                        SnackBar::new()
                            .message(tr!("Revert property failed") + " - " + &err.to_string()),
                    );
                }
                if self.reload_timeout.is_some() {
                    ctx.link().send_message(Msg::Load);
                }
            }
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
