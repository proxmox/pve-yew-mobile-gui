use proxmox_human_byte::HumanByte;
use proxmox_yew_comp::http_put;
use serde_json::Value;

use proxmox_schema::ApiType;

use pve_api_types::QemuConfigMemory;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Checkbox, FormContext, Hidden, Number};
use pwt::widget::Column;

use crate::form::{
    flatten_property_string, parse_property_string, property_string_from_parts, pspn,
};
use crate::widgets::{label_field, EditableProperty, RenderPropertyInputPanelFn};

// fixme: Number field changes type to Value::String on input!!
fn value_to_u64(value: Value) -> Option<u64> {
    match value {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn read_u64(form_ctx: &FormContext, name: &str) -> Option<u64> {
    let value = form_ctx.read().get_field_value(name.to_string());
    match value {
        Some(value) => value_to_u64(value),
        _ => None,
    }
}

fn input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, _| {
        let current_memory_prop = pspn("memory", "current");
        let current_memory = read_u64(&form_ctx, &current_memory_prop);

        let use_ballooning = form_ctx.read().get_field_checked("_use_ballooning");

        let disable_shares = {
            let balloon = read_u64(&form_ctx, "balloon");
            match (current_memory, balloon) {
                (Some(memory), Some(balloon)) => memory == balloon,
                _ => false,
            }
        };

        let memory_default = 512u64;

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(label_field(
                tr!("Memory") + " (MiB)",
                Number::<u64>::new()
                    .name(current_memory_prop)
                    .placeholder(memory_default.to_string())
                    .min(16)
                    .step(32),
            ))
            .with_child(Hidden::new().name("_old_memory").submit(false))
            .with_child(label_field(
                tr!("Minimum memory") + " (MiB)",
                Number::<u64>::new()
                    .name("balloon")
                    .submit_empty(true)
                    .disabled(!use_ballooning)
                    .min(1)
                    .max(current_memory)
                    .step(32)
                    .placeholder(current_memory.map(|n| n.to_string())),
            ))
            .with_child(label_field(
                tr!("Shares"),
                Number::<u64>::new()
                    .name("shares")
                    .submit_empty(true)
                    .disabled(!use_ballooning || disable_shares)
                    .placeholder(tr!("Default") + " (1000)")
                    .max(50000)
                    .step(10),
            ))
            .with_child(label_field(
                tr!("Ballooning Device"),
                Checkbox::new().name("_use_ballooning").submit(false),
            ))
            .into()
    })
}

pub fn qemu_memory_property(url: String) -> EditableProperty {
    EditableProperty::new("memory", tr!("Memory"))
        .loader(url.clone())
        .on_submit({
            let url = url.clone();
            move |data: Value| http_put(url.clone(), Some(data.clone()))
        })
        .required(true)
        .render_input_panel(input_panel())
        .renderer(|_, v, record| {
            let current = match v {
                Value::Null => 512,
                Value::String(s) => match parse_property_string::<QemuConfigMemory>(&s) {
                    Ok(parsed) => parsed.current,
                    Err(err) => {
                        log::error!("qemu_memory_property renderer: {err}");
                        return v.into();
                    }
                },
                _ => {
                    log::error!("qemu_memory_property renderer: got unexpected type");
                    return v.into();
                }
            };

            let balloon = record["balloon"].as_u64().unwrap_or(0);

            let current_hb = HumanByte::new_binary((current * 1024 * 1024) as f64);

            let text = if balloon == 0 {
                format!("{current_hb} [balloon=0]")
            } else {
                let balloon_hb = HumanByte::new_binary((balloon * 1024 * 1024) as f64);
                if current > balloon {
                    format!("{balloon_hb}/{current_hb}")
                } else {
                    current_hb.to_string()
                }
            };

            text.into()
        })
        .submit_hook(|form_ctx: FormContext| {
            let mut data = form_ctx.get_submit_data();

            if !form_ctx.read().get_field_checked("_use_ballooning") {
                data["balloon"] = Value::Null;
                data["shares"] = Value::Null;
            }

            property_string_from_parts::<QemuConfigMemory>(&mut data, "memory", true)?;
            data = delete_empty_values(&data, &["memory", "balloon", "shares"], false);
            Ok(data)
        })
        .load_hook(|mut record| {
            flatten_property_string(&mut record, "memory", &QemuConfigMemory::API_SCHEMA)?;
            let current_memory_prop = pspn("memory", "current");

            let use_ballooning = record["balloon"].as_u64().is_some();
            record["_use_ballooning"] = use_ballooning.into();

            if record["balloon"].is_null() {
                if let Some(current_memory) = record[current_memory_prop].as_u64() {
                    record["balloon"] = current_memory.into();
                    record["_old_memory"] = current_memory.into();
                }
            }
            Ok(record)
        })
        .on_change(|form_ctx: FormContext| {
            let current_memory_prop = pspn("memory", "current");
            let current_memory = form_ctx.read().get_field_value(current_memory_prop.clone());
            let old_memory = form_ctx.read().get_field_value("_old_memory");
            let balloon = form_ctx.read().get_field_value("balloon");

            match (&old_memory, &current_memory, &balloon) {
                (Some(old_memory), Some(current_memory), Some(balloon)) => {
                    if balloon == old_memory {
                        form_ctx
                            .write()
                            .set_field_value("balloon", current_memory.clone().into());
                    }
                }
                _ => {}
            }

            if let Some(current_memory) = current_memory {
                form_ctx
                    .write()
                    .set_field_value("_old_memory", current_memory.into());
            }
        })
}
