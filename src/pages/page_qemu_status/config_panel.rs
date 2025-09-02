use std::rc::Rc;

use pwt::props::{RenderFn, SubmitCallback};

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use proxmox_schema::{ApiType, ObjectSchema, Schema};

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Field, FormContext, Number};
use pwt::widget::Column;

use proxmox_yew_comp::{http_put, percent_encoding::percent_encode_component, SchemaValidation};

use pve_api_types::QemuConfig;

use crate::form::{
    load_property_string, submit_property_string, typed_load, QemuConfigOstypeSelector,
};
use crate::widgets::{EditableProperty, PropertyList};
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

pub struct PveQemuConfigPanel {
    properties: Rc<Vec<EditableProperty>>,
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

impl PveQemuConfigPanel {
    fn default_submit(props: &QemuConfigPanel) -> SubmitCallback<FormContext> {
        let url = get_config_url(&props.node, props.vmid);
        SubmitCallback::new(move |ctx: FormContext| {
            let value = ctx.get_submit_data();
            let value = delete_empty_values(&value, &["name", "ostype", "startup"], false);
            http_put(url.clone(), Some(value))
        })
    }

    fn properties(ctx: &Context<Self>) -> Rc<Vec<EditableProperty>> {
        fn render_string_input_panel(name: &'static str) -> RenderFn<FormContext> {
            RenderFn::new(move |_| {
                let mut input = Field::new().name(name.to_string()).submit_empty(true);

                if let Some((optional, schema)) = lookup_schema(&name) {
                    input.set_schema(schema);
                    input.set_required(!optional);
                }
                input.into()
            })
        }

        let props = ctx.props();
        let url = get_config_url(&props.node, props.vmid);

        Rc::new(vec![
            EditableProperty::new_bool("onboot", tr!("Start on boot"), false).required(true),
            EditableProperty::new_bool("tablet", tr!("Use tablet for pointer"), true)
                .required(true),
            EditableProperty::new_bool("acpi", tr!("ACPI support"), true).required(true),
            EditableProperty::new_bool("kvm", tr!("KVM hardware virtualization"), true)
                .required(true),
            EditableProperty::new_bool("freeze", tr!("Freeze CPU on startup"), false)
                .required(true),
            EditableProperty::new_bool("localtime", tr!("Use local time for RTC"), false)
                .required(true),
            EditableProperty::new_bool("protection", tr!("Protection"), false).required(true),
            EditableProperty::new("name", tr!("Name"))
                .required(true)
                .placeholder(format!("VM {}", props.vmid))
                .render_input_panel(render_string_input_panel("name")),
            EditableProperty::new("ostype", tr!("OS Type"))
                .required(true)
                .render_input_panel(move |_: &FormContext| {
                    QemuConfigOstypeSelector::new()
                        .style("width", "100%")
                        .name("ostype")
                        .submit_empty(true)
                        .into()
                }),
            EditableProperty::new("startup", tr!("Start/Shutdown order"))
                .required(true)
                .placeholder("order=any")
                .render_input_panel(move |_: &FormContext| {
                    Column::new()
                        .gap(2)
                        .class(pwt::css::Flex::Fill)
                        .class(pwt::css::AlignItems::Stretch)
                        .with_child(crate::widgets::label_field(
                            tr!("Order"),
                            Number::<u32>::new()
                                .name("_startup_order")
                                .placeholder(tr!("any")),
                        ))
                        .with_child(crate::widgets::label_field(
                            tr!("Startup delay"),
                            Number::<u32>::new()
                                .name("_startup_up")
                                .placeholder(tr!("default")),
                        ))
                        .with_child(crate::widgets::label_field(
                            tr!("Shutdown timeout"),
                            Number::<u32>::new()
                                .name("_startup_down")
                                .placeholder(tr!("default")),
                        ))
                        .into()
                })
                .loader(load_property_string::<QemuConfig, QemuConfigStartup>(
                    &url, "startup",
                ))
                .on_submit(Some(submit_property_string::<QemuConfigStartup>(
                    &url, "startup",
                ))),
            EditableProperty::new("boot", tr!("Boot Device")).required(true),
            EditableProperty::new("hotplug", tr!("Hotplug")).required(true),
            EditableProperty::new("startdate", tr!("RTC start date")).required(true),
            EditableProperty::new("smbios1", tr!("SMBIOS settings (type1)")).required(true),
            EditableProperty::new("agent", tr!("QEMU Guest Agent")).required(true),
            EditableProperty::new("spice-enhancements", tr!("Spice Enhancements")).required(true),
            EditableProperty::new("vmstatestorage", tr!("VM State Storage")).required(true),
            // Test
            //EditableProperty::new_bool("", tr!("")).required(true),
        ])
    }
}

impl Component for PveQemuConfigPanel {
    type Message = ();
    type Properties = QemuConfigPanel;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            properties: Self::properties(ctx),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let url = get_config_url(&props.node, props.vmid);
        let default_submit = Self::default_submit(props);

        PropertyList::new(Rc::clone(&self.properties))
            .loader(typed_load::<QemuConfig>(url))
            .on_submit(Some(default_submit))
            .into()
    }
}

impl From<QemuConfigPanel> for VNode {
    fn from(props: QemuConfigPanel) -> Self {
        let comp = VComp::new::<PveQemuConfigPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
