use std::rc::Rc;

use serde_json::Value;

use pwt::props::SubmitCallback;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::form::delete_empty_values;

use proxmox_yew_comp::{http_put, percent_encoding::percent_encode_component};

use pve_api_types::QemuConfig;

use crate::{
    form::typed_load,
    widgets::{EditableProperty, PropertyList},
};

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

impl PveQemuConfigPanel {
    fn default_submit(props: &QemuConfigPanel) -> SubmitCallback<Value> {
        let url = get_config_url(&props.node, props.vmid);
        SubmitCallback::new(move |value: Value| {
            let value = delete_empty_values(
                &value,
                &[
                    "name",
                    "boot",
                    "ostype",
                    "startup",
                    "hotplug",
                    "startdate",
                    "vmstatestorage",
                ],
                false,
            );
            http_put(url.clone(), Some(value))
        })
    }

    fn properties(ctx: &Context<Self>) -> Rc<Vec<EditableProperty>> {
        let props = ctx.props();

        Rc::new(vec![
            crate::form::qemu_onboot_property(),
            crate::form::qemu_tablet_property(),
            crate::form::qemu_acpi_property(),
            crate::form::qemu_kvm_property(),
            crate::form::qemu_freeze_property(),
            crate::form::qemu_localtime_property(),
            crate::form::qemu_protection_property(),
            crate::form::qemu_name_property(props.vmid),
            crate::form::qemu_ostype_property(),
            crate::form::qemu_startup_property(),
            crate::form::qemu_boot_property(),
            crate::form::qemu_hotplug_property(),
            crate::form::qemu_startdate_property(),
            crate::form::qemu_smbios_property("smbios1"),
            crate::form::qemu_agent_property("agent"),
            crate::form::qemu_spice_enhancement_property("spice_enhancements"),
            crate::form::qemu_vmstatestorage_property(&props.node),
            crate::form::qemu_amd_sev_property("amd-sev"),
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
