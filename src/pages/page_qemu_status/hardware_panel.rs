use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use proxmox_yew_comp::http_put;
use pwt::props::IntoOptionalInlineHtml;
use serde_json::Value;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::menu::{Menu, MenuButton, MenuItem};
use pwt::widget::{Fa, List, ListTile};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::{PveQmIde, PveQmIdeMedia, QemuConfig};

use crate::form::{
    qemu_bios_property, qemu_cpu_flags_property, qemu_display_property,
    qemu_kernel_scheduler_property, qemu_memory_property, qemu_sockets_cores_property, typed_load,
};
use crate::widgets::{icon_list_tile, EditDialog, EditableProperty};

#[derive(Clone, PartialEq, Properties)]
pub struct QemuHardwarePanel {
    vmid: u32,
    node: AttrValue,
}

impl QemuHardwarePanel {
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
    Dialog(Option<Html>),
    EditProperty(EditableProperty),
}

pub struct PveQemuHardwarePanel {
    data: Option<Result<QemuConfig, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    dialog: Option<Html>,
    memory_property: EditableProperty,
    bios_property: EditableProperty,
    sockets_cores_property: EditableProperty,
    cpu_flags_property: EditableProperty,
    kernel_scheduler_property: EditableProperty,
    display_property: EditableProperty,
}

impl PveQemuHardwarePanel {
    fn property_tile(
        &self,
        ctx: &Context<Self>,
        record: &Value,
        property: &EditableProperty,
        icon: Fa,
        trailing: impl IntoOptionalInlineHtml,
    ) -> ListTile {
        let name = &property.name.as_str();

        let value = match &record[name] {
            Value::Null => property
                .placeholder
                .clone()
                .unwrap_or(AttrValue::Static("-"))
                .to_string()
                .into(),
            other => property
                .renderer
                .clone()
                .unwrap()
                .apply(name, other, &record),
        };

        icon_list_tile(icon, property.title.clone(), value, trailing)
            .interactive(true)
            .on_activate(ctx.link().callback({
                let property = property.clone();
                move |_| Msg::EditProperty(property.clone())
            }))
    }

    fn processor_list_tile(&self, ctx: &Context<Self>, record: &Value) -> ListTile {
        let menu = Menu::new()
            .with_item(MenuItem::new(&self.sockets_cores_property.title).on_select(
                ctx.link().callback({
                    let property = self.sockets_cores_property.clone();
                    move |_| Msg::EditProperty(property.clone())
                }),
            ))
            .with_item(
                MenuItem::new(&self.kernel_scheduler_property.title).on_select(
                    ctx.link().callback({
                        let property = self.kernel_scheduler_property.clone();
                        move |_| Msg::EditProperty(property.clone())
                    }),
                ),
            )
            .with_item(MenuItem::new(&self.cpu_flags_property.title).on_select(
                ctx.link().callback({
                    let property = self.cpu_flags_property.clone();
                    move |_| Msg::EditProperty(property.clone())
                }),
            ));

        let menu_button: Html = MenuButton::new("")
            .class(pwt::css::ColorScheme::Neutral)
            .icon_class("fa fa-ellipsis-v fa-lg")
            .menu(menu)
            .into();

        let tile = self.property_tile(
            ctx,
            record,
            &self.sockets_cores_property,
            Fa::new("cpu"),
            menu_button,
        );

        tile
    }

    fn view_list(&self, ctx: &Context<Self>, data: &QemuConfig) -> Html {
        let record: Value = serde_json::to_value(data).unwrap();
        let mut list: Vec<ListTile> = Vec::new();

        let push_property_tile = |list: &mut Vec<_>, property, icon| {
            list.push(self.property_tile(ctx, &record, property, icon, ()));
        };

        push_property_tile(&mut list, &self.memory_property, Fa::new("memory"));
        list.push(self.processor_list_tile(ctx, &record));
        push_property_tile(&mut list, &self.bios_property, Fa::new("microchip"));
        push_property_tile(&mut list, &self.display_property, Fa::new("desktop"));

        list.push(icon_list_tile(
            Fa::new("gears"),
            tr!("Machine Type"),
            data.machine
                .as_ref()
                .map(|b| b.to_string())
                .unwrap_or(format!("{} (i440fx)", tr!("Default")))
                .to_string(),
            (),
        ));

        for (n, disk_config) in &data.ide {
            if let Ok(config) =
                proxmox_schema::property_string::parse::<PveQmIde>(disk_config.as_str())
            {
                if config.media == Some(PveQmIdeMedia::Cdrom) {
                    list.push(icon_list_tile(
                        Fa::new("cdrom"),
                        tr!("CD/DVD Drive") + &format!(" (ide{n})"),
                        disk_config.to_string(),
                        (),
                    ));
                } else {
                    list.push(icon_list_tile(
                        Fa::new("hdd-o"),
                        tr!("Hard Disk") + &format!(" (ide{n})"),
                        disk_config.to_string(),
                        (),
                    ));
                }
            }
        }

        for (n, disk_config) in &data.scsi {
            list.push(icon_list_tile(
                Fa::new("hdd-o"),
                tr!("Hard Disk") + &format!(" (scsi{n})"),
                disk_config.to_string(),
                (),
            ));
        }

        for (n, net_config) in &data.net {
            list.push(icon_list_tile(
                Fa::new("exchange"),
                tr!("Network Device") + &format!(" (net{n})"),
                net_config.to_string(),
                (),
            ));
        }

        List::new(list.len() as u64, move |pos| list[pos as usize].clone())
            .grid_template_columns("auto 1fr auto")
            .into()
    }
}

impl Component for PveQemuHardwarePanel {
    type Message = Msg;
    type Properties = QemuHardwarePanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
            reload_timeout: None,
            load_guard: None,
            dialog: None,
            memory_property: qemu_memory_property(),
            bios_property: qemu_bios_property(),
            sockets_cores_property: qemu_sockets_cores_property(),
            kernel_scheduler_property: qemu_kernel_scheduler_property(),
            cpu_flags_property: qemu_cpu_flags_property(),
            display_property: qemu_display_property(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.data = None;
        ctx.link().send_message(Msg::Load);
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            Msg::Dialog(dialog) => {
                if dialog.is_none() && self.dialog.is_some() {
                    ctx.link().send_message(Msg::Load);
                }
                self.dialog = dialog;
            }
            Msg::EditProperty(property) => {
                let url = get_config_url(&props.node, props.vmid);

                let dialog = EditDialog::from(property.clone())
                    .on_done(ctx.link().callback(|_| Msg::Dialog(None)))
                    .loader(typed_load::<QemuConfig>(url.clone()))
                    .on_submit({
                        let url = url.to_owned();
                        move |data: Value| http_put(url.clone(), Some(data.clone()))
                    })
                    .into();
                self.dialog = Some(dialog);
            }
            Msg::Load => {
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
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content =
            crate::widgets::render_loaded_data(&self.data, |data| self.view_list(ctx, data));
        crate::widgets::standard_card(tr!("Hardware"), None::<&str>)
            .min_height(200)
            .with_child(content)
            .with_optional_child(self.dialog.clone())
            .into()
    }
}

impl From<QemuHardwarePanel> for VNode {
    fn from(props: QemuHardwarePanel) -> Self {
        let comp = VComp::new::<PveQemuHardwarePanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
