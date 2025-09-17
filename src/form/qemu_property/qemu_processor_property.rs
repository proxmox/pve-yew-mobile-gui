use anyhow::Error;
use proxmox_schema::property_string::PropertyString;
use proxmox_schema::ApiType;

use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::Number;
use pwt::widget::Column;

use crate::{
    form::flatten_property_string,
    widgets::{label_field, EditableProperty, RenderPropertyInputPanelFn},
};

use pve_api_types::PveVmCpuConf;

const PROCESSOR_KEYS: &[&str] = &[
    "sockets", "cpu", "cores", "numa", "vcpus", "cpulimit", "cpuunits", "affinity",
];

fn renderer(_name: &str, value: &Value, record: &Value) -> Html {
    let cpu: Result<Option<PropertyString<PveVmCpuConf>>, _> =
        serde_json::from_value(record["cpu"].clone());

    let cputype = match &cpu {
        Ok(Some(cpu)) => cpu.cputype.as_deref().unwrap_or("kvm"),
        Ok(None) => "kvm",
        Err(_) => "unknown",
    };

    let cores = record["cores"].as_u64().unwrap_or(1);
    let sockets = record["sockets"].as_u64().unwrap_or(1);
    let count = sockets * cores;
    format!(
        "{count} ({}, {}) [{cputype}]",
        tr!("1 Core" | "{n} Cores" % cores),
        tr!("1 Socket" | "{n} Sockets" % sockets)
    )
    .into()
}

fn input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_, _| {
        Column::new()
            .gap(2)
            .with_child(label_field(
                tr!("Sockets"),
                Number::<u64>::new().name("sockets").min(1),
            ))
            .with_child(label_field(
                tr!("Cores"),
                Number::<u64>::new().name("cores").min(1),
            ))
            .into()
    })
}

pub fn qemu_processor_property() -> EditableProperty {
    EditableProperty::new("sockets", tr!("Processors"))
        .required(true)
        .renderer(renderer)
        .render_input_panel(input_panel())
        .load_hook(move |mut record: Value| {
            flatten_property_string(&mut record, "cpu", &PveVmCpuConf::API_SCHEMA)?;

            Ok(record)
        })
}
