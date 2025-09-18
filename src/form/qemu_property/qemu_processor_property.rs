use std::rc::Rc;

use serde_json::Value;

use proxmox_schema::{ApiType, ObjectSchemaType, Schema};

use pwt::prelude::*;
use pwt::props::PwtSpace;
use pwt::widget::form::{delete_empty_values, Checkbox, Field, FormContext, Hidden, Number};
use pwt::widget::{Column, Container, Row};

use crate::form::{property_string_from_parts, pspn, QemuCpuModelSelector};
use crate::{
    form::flatten_property_string,
    widgets::{label_field, EditableProperty, RenderPropertyInputPanelFn},
};

use pve_api_types::PveVmCpuConf;

//const PROCESSOR_KEYS: &[&str] = &[
//    "sockets", "cpu", "cores", "numa", "vcpus", "cpulimit", "cpuunits", "affinity",
//];

fn renderer(_name: &str, _value: &Value, record: &Value) -> Html {
    let cpu = record["cpu"].as_str().unwrap_or("kvm64");
    let cores = record["cores"].as_u64().unwrap_or(1);
    let sockets = record["sockets"].as_u64().unwrap_or(1);
    let count = sockets * cores;

    let mut text = format!(
        "{count} ({}, {}) [{cpu}]",
        tr!("1 Core" | "{n} Cores" % cores),
        tr!("1 Socket" | "{n} Sockets" % sockets)
    );

    if let Value::Bool(true) = record["numa"] {
        text += " [numa]";
    }

    if let Some(n) = &record["vcpus"].as_u64() {
        text += &format!(" [vcpus={n}]");
    }
    if let Some(n) = &record["cpulimit"].as_f64() {
        text += &format!(" [cpulimit={n}]");
    }
    if let Some(n) = &record["cpuunits"].as_u64() {
        text += &format!(" [cpuunits={n}]");
    }
    if let Some(s) = &record["affinity"].as_str() {
        text += &format!(" [affinity={s}]");
    }

    text.into()
}

fn add_hidden_cpu_properties(column: &mut Column, exclude: &[&str]) {
    // add unused cpu property - we want to keep them!
    match PveVmCpuConf::API_SCHEMA {
        Schema::Object(object_schema) => {
            let props = object_schema.properties();
            for (part, _, _) in props {
                if !exclude.contains(part) {
                    log::info!("ADD PART {part}");
                    column.add_child(Hidden::new().name(pspn("cpu", part)));
                }
            }
        }
        _ => {
            log::error!("property_string_from_parts: internal error - got unsupported schema type")
        }
    };
}

fn input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, _| {
        let total_cores;
        {
            let guard = form_ctx.read();
            let cores = guard
                .get_last_valid_value("cores")
                .unwrap_or(Value::Number(1.into()))
                .as_u64()
                .unwrap_or(1);
            let sockets = guard
                .get_last_valid_value("sockets")
                .unwrap_or(Value::Number(1.into()))
                .as_u64()
                .unwrap_or(1);
            total_cores = sockets * cores;
        }

        let mut column = Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .padding_top(2)
            .padding_bottom(1) // avoid scrollbar
            // QemuCpuModelSelector is scrollable, so we Diasble the SideDialog gesture detecture..
            .onpointerdown(|event: PointerEvent| {
                event.stop_propagation();
            })
            .ontouchstart(|event: TouchEvent| {
                event.stop_propagation();
            })
            .with_child(label_field(
                tr!("Type"),
                QemuCpuModelSelector::new()
                    .name(pspn("cpu", "cputype"))
                    .mobile(true),
            ))
            .with_child(Hidden::new().name("cpu")) // store original cpu settings
            .with_child(label_field(
                tr!("Sockets"),
                Number::<u64>::new().name("sockets").min(1),
            ))
            .with_child(label_field(
                tr!("Cores"),
                Number::<u64>::new().name("cores").min(1),
            ))
            .with_child(
                Row::new()
                    .padding_top(1)
                    .gap(PwtSpace::Em(0.5))
                    .with_child(tr!("Total cores") + ":")
                    .with_child(Container::new().with_child(total_cores.to_string())),
            );

        // add unused cpu property - we want to keep them!
        add_hidden_cpu_properties(&mut column, &["cputype"]);

        column.into()
    })
}

// Note: For the desktop view, we want everything in one edit wondow!
/*
fn input_panel_with_tabs() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, _| {
        let advanced = form_ctx.get_show_advanced();
        let total_cores;
        {
            let guard = form_ctx.read();
            let cores = guard
                .get_last_valid_value("cores")
                .unwrap_or(Value::Number(1.into()))
                .as_u64()
                .unwrap_or(1);
            let sockets = guard
                .get_last_valid_value("sockets")
                .unwrap_or(Value::Number(1.into()))
                .as_u64()
                .unwrap_or(1);
            total_cores = sockets * cores;
        }

        let main_view = Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .padding_top(2)
            .padding_bottom(1) // avoid scrollbar
            .with_child(label_field(tr!("Type"), Field::new().name("_cpu_cputype")))
            .with_child(label_field(
                tr!("Sockets"),
                Number::<u64>::new().name("sockets").min(1),
            ))
            .with_child(label_field(
                tr!("Cores"),
                Number::<u64>::new().name("cores").min(1),
            ))
            .with_child(
                Row::new()
                    .padding_top(1)
                    .gap(PwtSpace::Em(0.5))
                    .with_child(tr!("Total cores") + ":")
                    .with_child(Container::new().with_child(total_cores.to_string())),
            );

        let scheduler_view = Column::new()
            .gap(2)
            .padding_top(2)
            .padding_bottom(1) // avoid scrollbar
            .with_child("Scheduler...");

        let flags_view = Column::new()
            .gap(2)
            .padding_top(2)
            .padding_bottom(1) // avoid scrollbar
            .with_child("FLAGS...");

        TabPanel::new()
            .tab_bar_style(pwt::widget::TabBarStyle::Pills)
            .with_item(TabBarItem::new().key("main").label(tr!("Cores")), main_view)
            .with_item(
                TabBarItem::new()
                    .key("scheduler")
                    .label(tr!("Scheduler"))
                    .disabled(!advanced),
                scheduler_view,
            )
            .with_item(
                TabBarItem::new()
                    .key("flags")
                    .label(tr!("Flags"))
                    .disabled(!advanced),
                flags_view,
            )
            .into()
    })
}
*/

pub fn qemu_sockets_cores_property() -> EditableProperty {
    EditableProperty::new(
        "sockets",
        format!(
            "{}, {}, {}",
            tr!("Processor"),
            tr!("Sockets"),
            tr! {"Cores"}
        ),
    )
    .required(true)
    .renderer(renderer)
    .render_input_panel(input_panel())
    .load_hook(move |mut record: Value| {
        flatten_property_string(&mut record, "cpu", &PveVmCpuConf::API_SCHEMA)?;
        Ok(record)
    })
    .submit_hook(|form_ctx: FormContext| {
        let mut record = form_ctx.get_submit_data();
        property_string_from_parts::<PveVmCpuConf>(&mut record, "cpu", true)?;
        let record = delete_empty_values(&record, &["sockets", "cores", "cpu"], false);
        Ok(record)
    })
}

fn cpu_flags_input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_form_ctx: FormContext, _| {
        let mut column = Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .padding_top(2)
            .padding_bottom(1) // avoid scrollbar
            .with_child(Field::new().name(pspn("cpu", "flags")));

        // add unused cpu property - we want to keep them!
        add_hidden_cpu_properties(&mut column, &["flags"]);

        column.into()
    })
}

pub fn qemu_cpu_flags_property() -> EditableProperty {
    EditableProperty::new("cpu", tr!("CPU flags"))
        .required(true)
        .renderer(renderer)
        .render_input_panel(cpu_flags_input_panel())
        .load_hook(move |mut record: Value| {
            flatten_property_string(&mut record, "cpu", &PveVmCpuConf::API_SCHEMA)?;
            Ok(record)
        })
        .submit_hook(|form_ctx: FormContext| {
            let mut record = form_ctx.get_submit_data();
            property_string_from_parts::<PveVmCpuConf>(&mut record, "cpu", true)?;
            let record = delete_empty_values(&record, &["cpu"], false);
            Ok(record)
        })
}

fn kernel_scheduler_input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_form_ctx: FormContext, record: Rc<Value>| {
        let cores = record["cores"].as_u64().unwrap_or(1);
        let sockets = record["sockets"].as_u64().unwrap_or(1);
        let total_cores = cores * sockets;

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .padding_top(2)
            .padding_bottom(1) // avoid scrollbar
            .with_child(label_field(
                tr!("VCPUs"),
                Number::<u64>::new()
                    .name("vcpus")
                    .min(1)
                    .max(total_cores)
                    .placeholder(total_cores.to_string())
                    .submit_empty(true),
            ))
            .with_child(label_field(
                tr!("CPU units"),
                Number::<u64>::new()
                    .name("cpuunits")
                    .min(1)
                    .max(10000)
                    .placeholder("100")
                    .submit_empty(true),
            ))
            .with_child(label_field(
                tr!("CPU limit"),
                Number::<f64>::new()
                    .name("cpulimit")
                    .placeholder(tr!("unlimited"))
                    .min(0.0)
                    .max(128.0) // api maximum
                    .submit_empty(true),
            ))
            .with_child(label_field(
                tr!("CPU Affinity"),
                Field::new()
                    .name("affinity")
                    .placeholder(tr!("All Cores"))
                    .submit_empty(true),
            ))
            .with_child(
                Row::new()
                    .padding_top(1)
                    .class(pwt::css::AlignItems::Center)
                    .with_child(tr!("Enable NUMA"))
                    .with_flex_spacer()
                    .with_child(Checkbox::new().name("numa").switch(true)),
            )
            .into()
    })
}

pub fn qemu_kernel_scheduler_property() -> EditableProperty {
    EditableProperty::new("cpuunits", tr!("Kernel scheduler settings"))
        .required(true)
        .renderer(renderer)
        .render_input_panel(kernel_scheduler_input_panel())
        .submit_hook(|form_ctx: FormContext| {
            let mut record = form_ctx.get_submit_data();

            if let Some(cpulimit) = record["cpulimit"].as_f64() {
                if cpulimit == 0.0 {
                    record["cpulimit"] = Value::Null;
                }
            }

            let record = delete_empty_values(
                &record,
                &["vcpus", "cpuunits", "cpulimit", "affinity", "numa"],
                false,
            );
            Ok(record)
        })
}
