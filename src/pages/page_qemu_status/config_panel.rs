use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use pwt::touch::SideDialog;
use serde_json::Value;

use yew::html::Scope;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use proxmox_schema::{ApiType, Schema};

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Checkbox, Field, Form, FormContext, SubmitButton};
use pwt::widget::{Column, Container, List, ListTile, Row};

use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{
    http_get, http_put, percent_encoding::percent_encode_component, SchemaValidation,
};

use pve_api_types::QemuConfig;

use crate::form::QemuConfigOstypeSelector;
use crate::widgets::form_list_tile;

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

pub enum Msg {
    Load,
    LoadResult(Result<QemuConfig, Error>),
    EditBool(&'static str, AttrValue, bool),
    EditString(&'static str, AttrValue, String),
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
            if let Ok(ind) = object_schema
                .properties
                .binary_search_by_key(&name, |(n, _, _)| n)
            {
                let (_name, optional, schema) = object_schema.properties[ind];
                return Some((optional, schema));
            }
        }
    }
    None
}

impl PveQemuConfigPanel {
    fn changeable_config_bool(
        &self,
        ctx: &Context<Self>,
        title: impl Into<AttrValue>,
        name: &'static str,
        value: bool,
    ) -> ListTile {
        let title = title.into();

        let trailing = Container::new()
            .style("text-align", "end")
            .with_child(if value { tr!("Yes") } else { tr!("No") });
        form_list_tile(title.clone(), None::<&str>, Some(trailing.into()))
            .interactive(true)
            .onclick(
                ctx.link()
                    .callback(move |_| Msg::EditBool(name, title.clone(), value)),
            )
    }

    fn changeable_config_string(
        &self,
        ctx: &Context<Self>,
        title: impl Into<AttrValue>,
        name: &'static str,
        value: String,
    ) -> ListTile {
        let title = title.into();
        form_list_tile(title.clone(), Some(value.clone()), None)
            .interactive(true)
            .onclick(
                ctx.link()
                    .callback(move |_| Msg::EditString(name, title.clone(), value.clone())),
            )
    }

    fn edit_dialog(link: &Scope<Self>, input_panel: Html) -> SideDialog {
        let form = Form::new().class(pwt::css::FlexFit).with_child(
            Column::new()
                .class(pwt::css::FlexFit)
                .padding(2)
                .gap(4)
                .with_child(input_panel)
                .with_child(
                    Row::new().with_flex_spacer().with_child(
                        SubmitButton::new()
                            .text(tr!("Apply"))
                            .on_submit(link.callback(|ctx: FormContext| {
                                let value = ctx.get_submit_data();
                                Msg::Update(value)
                            })),
                    ),
                ),
        );

        SideDialog::new()
            .location(pwt::touch::SideDialogLocation::Bottom)
            .on_close(link.callback(|_| Msg::CloseDialog))
            .with_child(form)
    }

    fn edit_bool_config(
        &self,
        ctx: &Context<Self>,
        name: &str,
        title: &AttrValue,
        value: bool,
    ) -> SideDialog {
        let input = Checkbox::new()
            .name(name.to_string())
            .switch(true)
            .default(value);
        let panel = Row::new()
            .gap(1)
            .class(pwt::css::AlignItems::Center)
            .class("pwt-flex-fill")
            .class("pwt-font-size-title-medium")
            .with_child(title)
            .with_flex_spacer()
            .with_child(input);

        Self::edit_dialog(ctx.link(), panel.into())
    }

    fn edit_string_config(
        &self,
        ctx: &Context<Self>,
        name: &str,
        title: &AttrValue,
        value: String,
    ) -> SideDialog {
        let mut input = Field::new()
            .name(name.to_string())
            .default(value)
            .submit_empty(true);

        if let Some((optional, schema)) = lookup_schema(name) {
            input.set_schema(schema);
            input.set_required(!optional);
        }

        let panel = Column::new()
            .gap(1)
            .class("pwt-flex-fill")
            .class(pwt::css::AlignItems::Start)
            .class("pwt-font-size-title-medium")
            .with_child(title)
            .with_flex_spacer()
            .with_child(input);

        Self::edit_dialog(ctx.link(), panel.into())
    }

    fn edit_generic(
        &self,
        ctx: &Context<Self>,
        title: String,
        value: String,
        form: Html,
    ) -> ListTile {
        form_list_tile(title.clone(), value, None)
            .interactive(true)
            .onclick({
                let link = ctx.link().clone();
                let form = form.clone();
                move |_| {
                    let panel = Column::new()
                        .gap(1)
                        .class("pwt-flex-fill")
                        .class(pwt::css::AlignItems::Start)
                        .class("pwt-font-size-title-medium")
                        .with_child(title.clone())
                        .with_flex_spacer()
                        .with_child(form.clone());

                    link.send_message(Msg::ShowDialog(
                        Self::edit_dialog(&link, panel.into()).into(),
                    ));
                }
            })
    }

    fn view_config(&self, ctx: &Context<Self>, data: &QemuConfig) -> Html {
        let props = ctx.props();

        let mut list: Vec<ListTile> = Vec::new();

        list.push(self.changeable_config_bool(
            ctx,
            tr!("Start on boot"),
            "onboot",
            data.onboot.unwrap_or(false),
        ));
        list.push(self.changeable_config_bool(
            ctx,
            tr!("Use tablet for pointer"),
            "tablet",
            data.tablet.unwrap_or(true),
        ));
        list.push(self.changeable_config_bool(
            ctx,
            tr!("ACPI support"),
            "acpi",
            data.acpi.unwrap_or(true),
        ));
        list.push(self.changeable_config_bool(
            ctx,
            tr!("KVM hardware virtualization"),
            "kvm",
            true,
        ));
        list.push(self.changeable_config_bool(
            ctx,
            tr!("Freeze CPU on startup"),
            "freeze",
            data.freeze.unwrap_or(false),
        ));
        list.push(self.changeable_config_bool(
            ctx,
            tr!("Use local time for RTC"),
            "localtime",
            data.localtime.unwrap_or(false),
        ));
        list.push(self.changeable_config_bool(
            ctx,
            tr!("Protection"),
            "protection",
            data.protection.unwrap_or(false),
        ));

        list.push(
            self.changeable_config_string(
                ctx,
                tr!("Name"),
                "name",
                data.name
                    .as_ref()
                    .map(String::from)
                    .unwrap_or(format!("VM {}", props.vmid)),
            ),
        );

        list.push(form_list_tile(
            tr!("Start/Shutdown order"),
            data.startup
                .as_ref()
                .map(String::from)
                .unwrap_or(tr!("Default (any)")),
            None,
        ));

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
                QemuConfigOstypeSelector::new()
                    .style("width", "100%")
                    .name("ostype")
                    .submit_empty(true)
                    .default(value.map(|ostype| ostype.to_string()))
                    .into(),
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
            Msg::EditBool(name, title, value) => {
                let dialog = self.edit_bool_config(ctx, name, &title, value);
                self.edit_dialog = Some(dialog.into());
            }
            Msg::EditString(name, title, value) => {
                let dialog = self.edit_string_config(ctx, name, &title, value);
                self.edit_dialog = Some(dialog.into());
            }
            Msg::Update(value) => {
                self.edit_dialog = None;
                let link = ctx.link().clone();
                let url = get_config_url(&props.node, props.vmid);
                let value = delete_empty_values(&value, &["name", "ostype"], false);
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
