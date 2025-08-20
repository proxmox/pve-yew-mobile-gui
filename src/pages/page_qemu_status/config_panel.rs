use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use pwt::touch::SideDialog;
use serde_json::{json, Value};

use yew::html::Scope;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use proxmox_schema::{ApiType, ObjectSchema, Schema};

use pwt::prelude::*;
use pwt::widget::form::{
    delete_empty_values, Checkbox, Field, Form, FormContext, Number, ResetButton, SubmitButton,
};
use pwt::widget::{Column, Container, List, ListTile, Row};

use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{
    form::property_string_from_parts, http_get, http_put,
    percent_encoding::percent_encode_component, SchemaValidation,
};

use pve_api_types::QemuConfig;

use crate::form::QemuConfigOstypeSelector;
use crate::widgets::form_list_tile;
use crate::QemuConfigStartup;

#[derive(Clone, PartialEq, Properties)]
pub struct QemuConfigPanel {
    vmid: u32,
    node: AttrValue,
}

impl QemuConfigPanel {
    pub fn new(node: impl Into<AttrValue>, vmid: u32) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

fn get_config_url(node: &str, vmid: u32) -> String {
    format!(
        "/nodes/{}/qemu/{}/config",
        percent_encode_component(node),
        vmid
    )
}
// fixme: use proxmox_yew_comp::flatten_property_string instead
// Split property string into separate parts
pub fn property_string_values(
    property: Option<String>,
    name: &str,
    schema: &'static Schema,
) -> Value {
    let mut data = json!({});
    if let Some(prop_str) = &property {
        match schema.parse_property_string(&prop_str) {
            Ok(value) => {
                if let Value::Object(map) = value {
                    for (k, v) in map {
                        data[format!("_{name}_{k}")] = v;
                    }
                }
            }
            Err(err) => {
                log::error!("unable to parse property {name}: {err}");
            }
        }
    }
    data
}

pub enum Msg {
    Load,
    LoadResult(Result<QemuConfig, Error>),
    Update(Value),
    UpdateResult(Result<(), Error>),
    CloseDialog,
    ShowDialog(Html),
}

pub struct PveQemuConfigPanel {
    data: Option<Result<QemuConfig, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    store_guard: Option<AsyncAbortGuard>,
    edit_dialog: Option<Html>,
}

fn lookup_schema(name: &str) -> Option<(bool, &'static Schema)> {
    let allof_schema = QemuConfig::API_SCHEMA.unwrap_all_of_schema();

    for entry in allof_schema.list {
        if let Schema::Object(object_schema) = entry {
            if let Some((optional, schema)) = lookup_object_property_schema(&object_schema, name) {
                return Some((optional, schema));
            }
        }
    }
    None
}

fn lookup_object_property_schema(
    object_schema: &ObjectSchema,
    name: &str,
) -> Option<(bool, &'static Schema)> {
    if let Ok(ind) = object_schema
        .properties
        .binary_search_by_key(&name, |(n, _, _)| n)
    {
        let (_name, optional, schema) = object_schema.properties[ind];
        return Some((optional, schema));
    }
    None
}

/*
fn unwrap_object_property_schema(object_schema: &ObjectSchema, name: &str) -> &'static Schema {
    lookup_object_property_schema(object_schema, name)
        .unwrap()
        .1
}
*/

impl PveQemuConfigPanel {
    fn edit_bool(&self, ctx: &Context<Self>, name: &str, title: String, value: bool) -> ListTile {
        let trailing = Container::new()
            .style("text-align", "end")
            .with_child(if value { tr!("Yes") } else { tr!("No") });

        form_list_tile(title.clone(), None::<&str>, Some(trailing.into()))
            .interactive(true)
            .onclick({
                let link = ctx.link().clone();
                let input = Checkbox::new()
                    .name(name.to_string())
                    .switch(true)
                    .default(value);
                move |_| {
                    let panel = Row::new()
                        .gap(1)
                        .class(pwt::css::AlignItems::Center)
                        .class("pwt-flex-fill")
                        .class("pwt-font-size-title-medium")
                        .with_child(&title)
                        .with_flex_spacer()
                        .with_child(input.clone());

                    link.send_message(Msg::ShowDialog(
                        Self::edit_dialog(&link, panel.into(), None).into(),
                    ));
                }
            })
    }

    fn edit_dialog(
        link: &Scope<Self>,
        input_panel: Html,
        on_submit: Option<Callback<FormContext>>,
    ) -> SideDialog {
        let on_submit = match on_submit {
            Some(on_submit) => on_submit,
            None => link.callback(|ctx: FormContext| {
                let value = ctx.get_submit_data();
                Msg::Update(value)
            }),
        };

        let form = Form::new().class(pwt::css::FlexFit).with_child(
            Column::new()
                .class(pwt::css::FlexFit)
                .padding(2)
                .gap(4)
                .with_child(input_panel)
                .with_child(
                    Row::new()
                        .gap(2)
                        .with_flex_spacer()
                        .with_child(ResetButton::new().class("pwt-button-text"))
                        .with_child(SubmitButton::new().text(tr!("Update")).on_submit(on_submit)),
                ),
        );

        SideDialog::new()
            .location(pwt::touch::SideDialogLocation::Bottom)
            .on_close(link.callback(|_| Msg::CloseDialog))
            .with_child(form)
    }

    fn edit_string(
        &self,
        ctx: &Context<Self>,
        name: &str,
        title: String,
        value: String,
    ) -> ListTile {
        self.edit_generic(
            ctx,
            title,
            value.clone(),
            || {
                let mut input = Field::new()
                    .name(name.to_string())
                    .default(value.clone())
                    .submit_empty(true);

                if let Some((optional, schema)) = lookup_schema(name) {
                    input.set_schema(schema);
                    input.set_required(!optional);
                }
                input.into()
            },
            None,
        )
    }

    fn edit_generic(
        &self,
        ctx: &Context<Self>,
        title: String,
        value: String,
        create_form: impl Fn() -> Html,
        on_submit: Option<Callback<FormContext>>,
    ) -> ListTile {
        form_list_tile(title.clone(), value, None)
            .interactive(true)
            .onclick({
                let link = ctx.link().clone();
                let form = create_form();
                move |_| {
                    let panel = Column::new()
                        .gap(1)
                        .class(pwt::css::Flex::Fill)
                        .class(pwt::css::AlignItems::Stretch)
                        .class("pwt-font-size-title-medium")
                        .with_child(title.clone())
                        .with_flex_spacer()
                        .with_child(form.clone());

                    link.send_message(Msg::ShowDialog(
                        Self::edit_dialog(&link, panel.into(), on_submit.clone()).into(),
                    ));
                }
            })
    }

    fn view_config(&self, ctx: &Context<Self>, data: &QemuConfig) -> Html {
        let props = ctx.props();

        let mut list: Vec<ListTile> = Vec::new();

        list.push(self.edit_bool(
            ctx,
            "onboot",
            tr!("Start on boot"),
            data.onboot.unwrap_or(false),
        ));
        list.push(self.edit_bool(
            ctx,
            "tablet",
            tr!("Use tablet for pointer"),
            data.tablet.unwrap_or(true),
        ));
        list.push(self.edit_bool(ctx, "acpi", tr!("ACPI support"), data.acpi.unwrap_or(true)));
        list.push(self.edit_bool(ctx, "kvm", tr!("KVM hardware virtualization"), true));
        list.push(self.edit_bool(
            ctx,
            "freeze",
            tr!("Freeze CPU on startup"),
            data.freeze.unwrap_or(false),
        ));
        list.push(self.edit_bool(
            ctx,
            "localtime",
            tr!("Use local time for RTC"),
            data.localtime.unwrap_or(false),
        ));
        list.push(self.edit_bool(
            ctx,
            "protection",
            tr!("Protection"),
            data.protection.unwrap_or(false),
        ));

        list.push(self.edit_string(
            ctx,
            "name",
            tr!("Name"),
            data.name.clone().unwrap_or(format!("VM {}", props.vmid)),
        ));

        list.push({
            let value = data.startup.clone();
            let data =
                property_string_values(value.clone(), "startup", &QemuConfigStartup::API_SCHEMA);

            /*
                        let order_schema = unwrap_object_property_schema(
                            QemuConfigStartup::API_SCHEMA.unwrap_object_schema(),
                            "order",
                        );
                        let up_schema = unwrap_object_property_schema(
                            QemuConfigStartup::API_SCHEMA.unwrap_object_schema(),
                            "up",
                        );
                        let down_schema = unwrap_object_property_schema(
                            QemuConfigStartup::API_SCHEMA.unwrap_object_schema(),
                            "down",
                        );
            */

            self.edit_generic(
                ctx,
                tr!("Start/Shutdown order"),
                value.unwrap_or(String::from(tr!("Default") + " (" + &tr!("any") + ")")),
                move || {
                    Column::new()
                        .gap(2)
                        .class(pwt::css::Flex::Fill)
                        .class(pwt::css::AlignItems::Stretch)
                        .with_child(crate::widgets::label_field(
                            tr!("Order"),
                            Number::<u32>::new()
                                .name("_startup_order")
                                .placeholder(tr!("any"))
                                .default(data["_startup_order"].as_u64().map(|n| n as u32)), //.schema(order_schema),
                        ))
                        .with_child(crate::widgets::label_field(
                            tr!("Startup delay"),
                            Number::<u32>::new()
                                .name("_startup_up")
                                .placeholder(tr!("default"))
                                .default(data["_startup_up"].as_u64().map(|n| n as u32)), //.schema(up_schema),
                        ))
                        .with_child(crate::widgets::label_field(
                            tr!("Shutdown timeout"),
                            Number::<u32>::new()
                                .name("_startup_down")
                                .placeholder(tr!("default"))
                                .default(data["_startup_down"].as_i64().map(|n| n as u32)), //.schema(down_schema),
                        ))
                        .into()
                },
                Some(ctx.link().callback(|ctx: FormContext| {
                    let mut value = ctx.get_submit_data();
                    property_string_from_parts::<QemuConfigStartup>(&mut value, "startup", true);
                    Msg::Update(value)
                })),
            )
        });

        list.push({
            let value = data.ostype;
            let value_str = match value {
                Some(ostype) => QemuConfigOstypeSelector::render_value(&ostype.to_string()),
                None => String::from("-"),
            };

            self.edit_generic(
                ctx,
                tr!("OS Type"),
                value_str,
                || {
                    QemuConfigOstypeSelector::new()
                        .style("width", "100%")
                        .name("ostype")
                        .submit_empty(true)
                        .default(value.map(|ostype| ostype.to_string()))
                        .into()
                },
                None,
            )
        });

        list.push(form_list_tile(
            tr!("Boot Device"),
            data.boot
                .as_ref()
                .map(String::from)
                .unwrap_or(tr!("Disk, Network, USB")),
            None,
        ));

        list.push(form_list_tile(
            tr!("Hotplug"),
            data.hotplug
                .as_ref()
                .map(String::from)
                .unwrap_or(String::from("disk,network,usb")),
            None,
        ));

        list.push(form_list_tile(
            tr!("RTC start date"),
            data.startdate
                .as_ref()
                .map(String::from)
                .unwrap_or(String::from("now")),
            None,
        ));

        list.push(form_list_tile(
            tr!("SMBIOS settings (type1)"),
            data.smbios1
                .as_ref()
                .map(String::from)
                .unwrap_or(String::from("-")),
            None,
        ));

        list.push(form_list_tile(
            tr!("QEMU Guest Agent"),
            data.smbios1
                .as_ref()
                .map(String::from)
                .unwrap_or(tr!("Default (disabled)")),
            None,
        ));

        list.push(form_list_tile(
            tr!("Spice Enhancements"),
            data.spice_enhancements
                .as_ref()
                .map(String::from)
                .unwrap_or(tr!("No enhancements")),
            None,
        ));

        list.push(form_list_tile(
            tr!("VM State Storage"),
            data.vmstatestorage
                .as_ref()
                .map(String::from)
                .unwrap_or(String::from("1 (autogenerated)")),
            None,
        ));

        Container::new()
            .class(pwt::css::FlexFit)
            .with_child(
                List::new(list.len() as u64, move |pos| list[pos as usize].clone())
                    .class(pwt::css::FlexFit)
                    .grid_template_columns("1fr auto"),
            )
            .with_optional_child(self.edit_dialog.clone())
            .into()
    }
}

impl Component for PveQemuConfigPanel {
    type Message = Msg;
    type Properties = QemuConfigPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
            reload_timeout: None,
            load_guard: None,
            store_guard: None,
            edit_dialog: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Load => {
                self.reload_timeout = None;
                let link = ctx.link().clone();
                let url = get_config_url(&props.node, props.vmid);
                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&url, None).await;
                    link.send_message(Msg::LoadResult(result));
                }));
            }
            Msg::LoadResult(result) => {
                self.data = Some(result.map_err(|err| err.to_string()));
                let link = ctx.link().clone();
                self.reload_timeout = Some(Timeout::new(3000, move || {
                    link.send_message(Msg::Load);
                }));
            }
            Msg::UpdateResult(result) => {
                if self.reload_timeout.is_some() {
                    ctx.link().send_message(Msg::Load);
                }
                if let Err(err) = result {
                    crate::show_failed_command_error(ctx.link(), err);
                }
            }
            Msg::ShowDialog(dialog) => {
                self.edit_dialog = Some(dialog);
            }
            Msg::CloseDialog => {
                self.edit_dialog = None;
            }
            Msg::Update(value) => {
                self.edit_dialog = None;
                let link = ctx.link().clone();
                let url = get_config_url(&props.node, props.vmid);
                let value = delete_empty_values(&value, &["name", "ostype", "startup"], false);
                self.store_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result: Result<(), Error> = http_put(&url, Some(value)).await;
                    link.send_message(Msg::UpdateResult(result));
                }));
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        crate::widgets::render_loaded_data(&self.data, |data| self.view_config(ctx, data))
    }
}

impl From<QemuConfigPanel> for VNode {
    fn from(props: QemuConfigPanel) -> Self {
        let comp = VComp::new::<PveQemuConfigPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
