use std::rc::Rc;

use serde_json::Value;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;

use proxmox_yew_comp::form::typed_load;
use proxmox_yew_comp::pending_property_view::PendingPropertyList;
use proxmox_yew_comp::EditableProperty;
use proxmox_yew_comp::{http_put, percent_encoding::percent_encode_component};

use pve_api_types::QemuConfig;

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

pub struct PveQemuConfigPanel {
    properties: Rc<Vec<EditableProperty>>,
}

impl PveQemuConfigPanel {
    fn properties(ctx: &Context<Self>) -> Rc<Vec<EditableProperty>> {
        let props = ctx.props();

        let mobile = true;

        Rc::new(vec![
            proxmox_yew_comp::form::pve::qemu_onboot_property(mobile),
            proxmox_yew_comp::form::pve::qemu_tablet_property(mobile),
            proxmox_yew_comp::form::pve::qemu_acpi_property(mobile),
            proxmox_yew_comp::form::pve::qemu_kvm_property(mobile),
            proxmox_yew_comp::form::pve::qemu_freeze_property(mobile),
            proxmox_yew_comp::form::pve::qemu_localtime_property(mobile),
            proxmox_yew_comp::form::pve::qemu_protection_property(mobile),
            proxmox_yew_comp::form::pve::qemu_name_property(props.vmid, mobile),
            proxmox_yew_comp::form::pve::qemu_ostype_property(mobile),
            proxmox_yew_comp::form::pve::qemu_startup_property(mobile),
            proxmox_yew_comp::form::pve::qemu_boot_property(mobile),
            proxmox_yew_comp::form::pve::qemu_hotplug_property(),
            proxmox_yew_comp::form::pve::qemu_startdate_property(mobile),
            proxmox_yew_comp::form::pve::qemu_smbios_property(mobile),
            proxmox_yew_comp::form::pve::qemu_agent_property(),
            proxmox_yew_comp::form::pve::qemu_spice_enhancement_property(),
            proxmox_yew_comp::form::pve::qemu_vmstatestorage_property(&props.node, mobile),
            proxmox_yew_comp::form::pve::qemu_amd_sev_property(mobile),
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
        let editor_url = format!(
            "/nodes/{}/qemu/{}/config",
            percent_encode_component(&props.node),
            props.vmid
        );
        let pending_url = format!(
            "/nodes/{}/qemu/{}/pending",
            percent_encode_component(&props.node),
            props.vmid
        );
        PendingPropertyList::new(Rc::clone(&self.properties))
            .class(pwt::css::FlexFit)
            .pending_loader(pending_url)
            .editor_loader(typed_load::<QemuConfig>(editor_url.clone()))
            .on_submit(move |value: Value| http_put(editor_url.clone(), Some(value)))
            .into()
    }
}

impl From<QemuConfigPanel> for VNode {
    fn from(props: QemuConfigPanel) -> Self {
        let comp = VComp::new::<PveQemuConfigPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
