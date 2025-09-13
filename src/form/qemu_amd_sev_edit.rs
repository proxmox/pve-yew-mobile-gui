use pve_api_types::{PveQemuSevFmt, PveQemuSevFmtType};

use pwt::prelude::*;

use crate::widgets::EditableProperty;

pub fn qemu_amd_sev_property(name: impl Into<String>, url: impl Into<String>) -> EditableProperty {
    let url = url.into();
    let name = name.into();

    EditableProperty::new("amd-sev", tr!("AMD SEV"))
        .required(true)
        .placeholder(format!("{} ({})", tr!("Default"), tr!("Disabled")))
        .renderer(|_, v, _| {
            if let Ok(data) = crate::form::parse_property_string_value::<PveQemuSevFmt>(v) {
                match data.ty {
                    PveQemuSevFmtType::Std => "AMD SEV",
                    PveQemuSevFmtType::Es => "AMD SEV-ES",
                    PveQemuSevFmtType::Snp => "AMD SEV-SNP",
                }
                .into()
            } else {
                v.into()
            }
        })
}
