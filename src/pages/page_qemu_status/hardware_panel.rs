use std::collections::HashSet;
use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use proxmox_schema::property_string::PropertyString;
use proxmox_yew_comp::{http_put, SafeConfirmDialog};
use serde_json::{json, Value};

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::widget::menu::{Menu, MenuButton, MenuItem};
use pwt::widget::{Fa, List, ListTile};
use pwt::AsyncAbortGuard;
use pwt::{prelude::*, AsyncPool};

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};
use pwt::props::{IntoOptionalInlineHtml, SubmitCallback};

use pve_api_types::{
    PveQmIde, PveQmIdeMedia, QemuConfig, QemuConfigIdeArray, QemuConfigNetArray, QemuConfigSata,
    QemuConfigSataArray, QemuConfigScsi, QemuConfigScsiArray, QemuConfigVirtioArray,
};

use crate::api_types::QemuPendingConfigValue;
use crate::form::{
    qemu_bios_property, qemu_cdrom_property, qemu_cpu_flags_property, qemu_disk_property,
    qemu_display_property, qemu_kernel_scheduler_property, qemu_machine_property,
    qemu_memory_property, qemu_network_mtu_property, qemu_network_property, qemu_scsihw_property,
    qemu_sockets_cores_property, qemu_vmstate_property, typed_load,
};
use crate::widgets::{
    pve_pending_config_array_to_objects, EditDialog, EditableProperty, PendingPropertyList,
};

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

    fn editor_url(&self) -> String {
        format!(
            "/nodes/{}/qemu/{}/config",
            percent_encode_component(&self.node),
            self.vmid
        )
    }

    fn pending_url(&self) -> String {
        format!(
            "/nodes/{}/qemu/{}/pending",
            percent_encode_component(&self.node),
            self.vmid
        )
    }
}

pub enum Msg {
    Load,
    LoadResult(Result<Vec<QemuPendingConfigValue>, Error>),
    Dialog(Option<Html>),
    EditProperty(EditableProperty),
    Revert(EditableProperty),
    CommandResult(Result<(), Error>),
    DeleteDevice(String),
}

pub struct PveQemuHardwarePanel {
    data: Option<Result<(Value, Value, HashSet<String>), String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    async_pool: AsyncPool,
    dialog: Option<Html>,

    memory_property: EditableProperty,
    bios_property: EditableProperty,
    sockets_cores_property: EditableProperty,
    cpu_flags_property: EditableProperty,
    kernel_scheduler_property: EditableProperty,
    display_property: EditableProperty,
    machine_property: EditableProperty,
    scsihw_property: EditableProperty,
    vmstate_property: EditableProperty,

    on_submit: SubmitCallback<Value>,
}

impl PveQemuHardwarePanel {
    fn property_tile(
        &self,
        ctx: &Context<Self>,
        current: &Value,
        pending: &Value,
        property: EditableProperty,
        icon: Fa,
        trailing: impl IntoOptionalInlineHtml,
    ) -> ListTile {
        let on_revert = Callback::from({
            let property = property.clone();
            ctx.link()
                .callback(move |_: Event| Msg::Revert(property.clone()))
        });

        let list_tile = PendingPropertyList::render_icon_list_tile(
            current, pending, &property, icon, trailing, on_revert,
        );

        list_tile
            .interactive(true)
            .on_activate(ctx.link().callback({
                let property = property.clone();
                move |_| Msg::EditProperty(property.clone())
            }))
    }

    fn processor_list_tile(
        &self,
        ctx: &Context<Self>,
        record: &Value,
        pending: &Value,
    ) -> ListTile {
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
            .class("circle")
            .icon_class("fa fa-ellipsis-v fa-lg")
            .menu(menu)
            .into();

        let tile = self.property_tile(
            ctx,
            record,
            pending,
            self.sockets_cores_property.clone(),
            Fa::new("cpu"),
            menu_button,
        );

        tile
    }

    fn network_list_tile(
        &self,
        ctx: &Context<Self>,
        name: &str,
        record: &Value,
        pending: &Value,
    ) -> ListTile {
        let props = ctx.props();
        let network_property =
            qemu_network_property(Some(name.to_string()), Some(props.node.clone()));
        let mtu_property =
            qemu_network_mtu_property(Some(name.to_string()), Some(props.node.clone()));

        let menu = Menu::new()
            .with_item(
                MenuItem::new(&network_property.title).on_select(ctx.link().callback({
                    let property = network_property.clone();
                    move |_| Msg::EditProperty(property.clone())
                })),
            )
            .with_item(
                MenuItem::new(&mtu_property.title).on_select(ctx.link().callback({
                    let property = mtu_property.clone();
                    move |_| Msg::EditProperty(property.clone())
                })),
            )
            .with_item({
                let link = ctx.link().clone();
                let dialog: Html = SafeConfirmDialog::new(name.to_string())
                    .on_done(link.callback(|_| Msg::Dialog(None)))
                    .on_confirm(link.callback({
                        let name = name.to_string();
                        move |_| Msg::DeleteDevice(name.clone())
                    }))
                    .into();
                MenuItem::new(tr!("Delete device")).on_select(
                    ctx.link()
                        .callback(move |_| Msg::Dialog(Some(dialog.clone()))),
                )
            });

        let menu_button: Html = MenuButton::new("")
            .class(pwt::css::ColorScheme::Neutral)
            .class("circle")
            .icon_class("fa fa-ellipsis-v fa-lg")
            .menu(menu)
            .into();

        let tile = self.property_tile(
            ctx,
            record,
            pending,
            network_property,
            Fa::new("exchange"),
            menu_button,
        );

        tile
    }

    fn view_list(
        &self,
        ctx: &Context<Self>,
        (record, pending, keys): &(Value, Value, HashSet<String>),
    ) -> Html {
        let props = ctx.props();
        let mut list: Vec<ListTile> = Vec::new();

        let push_property_tile = |list: &mut Vec<_>, property: EditableProperty, icon| {
            let name = match property.get_name() {
                Some(name) => name.to_string(),
                None::<_> => return,
            };

            if property.required || keys.contains(&name) {
                let mut tile = self.property_tile(ctx, &record, &pending, property, icon, ());
                tile.set_key(name);
                list.push(tile);
            }
        };

        push_property_tile(&mut list, self.memory_property.clone(), Fa::new("memory"));
        list.push(self.processor_list_tile(ctx, &record, &pending));
        push_property_tile(&mut list, self.bios_property.clone(), Fa::new("microchip"));
        push_property_tile(&mut list, self.display_property.clone(), Fa::new("desktop"));
        push_property_tile(&mut list, self.machine_property.clone(), Fa::new("cogs"));
        push_property_tile(&mut list, self.scsihw_property.clone(), Fa::new("database"));

        // fixme: this should be removable - add menu with delete
        push_property_tile(
            &mut list,
            self.vmstate_property.clone(),
            Fa::new("download"),
        );

        for n in 0..QemuConfigIdeArray::MAX {
            let name = format!("ide{n}");
            if !keys.contains(&name) {
                continue;
            }
            let media = match serde_json::from_value::<Option<PropertyString<PveQmIde>>>(
                pending[&name].clone(),
            ) {
                Ok(Some(ide)) => ide.media.unwrap_or(PveQmIdeMedia::Disk),
                Ok(None) => PveQmIdeMedia::Disk,
                Err(err) => {
                    log::error!("unable to parse drive '{name}' media: {err}");
                    continue;
                }
            };
            if media == PveQmIdeMedia::Cdrom {
                let property = qemu_cdrom_property(Some(name.clone()), Some(props.node.clone()));
                push_property_tile(&mut list, property, Fa::new("cdrom"));
            } else {
                let property = qemu_disk_property(Some(name.clone()), Some(props.node.clone()));
                push_property_tile(&mut list, property, Fa::new("hdd-o"));
            }
        }

        for n in 0..QemuConfigSataArray::MAX {
            let name = format!("sata{n}");
            if !keys.contains(&name) {
                continue;
            }
            let media = match serde_json::from_value::<Option<PropertyString<QemuConfigSata>>>(
                pending[&name].clone(),
            ) {
                Ok(Some(ide)) => ide.media.unwrap_or(PveQmIdeMedia::Disk),
                Ok(None) => PveQmIdeMedia::Disk,
                Err(err) => {
                    log::error!("unable to parse drive '{name}' media: {err}");
                    continue;
                }
            };
            if media == PveQmIdeMedia::Cdrom {
                let property = qemu_cdrom_property(Some(name.clone()), Some(props.node.clone()));
                push_property_tile(&mut list, property, Fa::new("cdrom"));
            } else {
                let property = qemu_disk_property(Some(name.clone()), Some(props.node.clone()));
                push_property_tile(&mut list, property, Fa::new("hdd-o"));
            }
        }

        for n in 0..QemuConfigScsiArray::MAX {
            let name = format!("scsi{n}");
            if !keys.contains(&name) {
                continue;
            }
            let media = match serde_json::from_value::<Option<PropertyString<QemuConfigScsi>>>(
                pending[&name].clone(),
            ) {
                Ok(Some(scsi)) => scsi.media.unwrap_or(PveQmIdeMedia::Disk),
                Ok(None) => PveQmIdeMedia::Disk,
                Err(err) => {
                    log::error!("unable to parse drive '{name}' media: {err}");
                    continue;
                }
            };
            if media == PveQmIdeMedia::Cdrom {
                let property = qemu_cdrom_property(Some(name.clone()), Some(props.node.clone()));
                push_property_tile(&mut list, property, Fa::new("cdrom"));
            } else {
                let property = qemu_disk_property(Some(name.clone()), Some(props.node.clone()));
                push_property_tile(&mut list, property, Fa::new("hdd-o"));
            }
        }

        for n in 0..QemuConfigVirtioArray::MAX {
            let name = format!("virtio{n}");
            if !keys.contains(&name) {
                continue;
            }
            let property = qemu_disk_property(Some(name.clone()), Some(props.node.clone()));
            push_property_tile(&mut list, property, Fa::new("hdd-o"));
        }

        for n in 0..QemuConfigNetArray::MAX {
            let name = format!("net{n}");
            if !keys.contains(&name) {
                continue;
            }
            list.push(self.network_list_tile(ctx, &name, &record, &pending));
        }

        List::new(list.len() as u64, move |pos| list[pos as usize].clone())
            .grid_template_columns("auto 1fr")
            .into()
    }

    fn create_on_submit(props: &QemuHardwarePanel) -> SubmitCallback<Value> {
        let url = props.editor_url();
        SubmitCallback::new(move |data: Value| http_put(url.clone(), Some(data.clone())))
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
            async_pool: AsyncPool::new(),
            dialog: None,
            memory_property: qemu_memory_property(),
            bios_property: qemu_bios_property(),
            sockets_cores_property: qemu_sockets_cores_property(),
            kernel_scheduler_property: qemu_kernel_scheduler_property(),
            cpu_flags_property: qemu_cpu_flags_property(),
            display_property: qemu_display_property(),
            machine_property: qemu_machine_property(),
            scsihw_property: qemu_scsihw_property(),
            vmstate_property: qemu_vmstate_property(),

            on_submit: Self::create_on_submit(ctx.props()),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.data = None;
        self.on_submit = Self::create_on_submit(ctx.props());
        ctx.link().send_message(Msg::Load);
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            Msg::DeleteDevice(name) => {
                let link = ctx.link().clone();
                let on_submit = self.on_submit.clone();
                let param = json!({ "delete": [name] });
                self.async_pool.spawn(async move {
                    let result = on_submit.apply(param).await;
                    link.send_message(Msg::CommandResult(result));
                });
            }
            Msg::Revert(property) => {
                let link = ctx.link().clone();
                let keys = match property.revert_keys.as_deref() {
                    Some(keys) => keys.iter().map(|a| a.to_string()).collect(),
                    None::<_> => {
                        if let Some(name) = property.get_name() {
                            vec![name.to_string()]
                        } else {
                            log::error!("hardware panel: cannot revert property without name",);
                            return false;
                        }
                    }
                };
                let on_submit = self.on_submit.clone();
                let param = json!({ "revert": keys });
                self.async_pool.spawn(async move {
                    let result = on_submit.apply(param).await;
                    link.send_message(Msg::CommandResult(result));
                });
            }
            Msg::CommandResult(result) => {
                if let Err(err) = result {
                    crate::show_failed_command_error(ctx.link(), err);
                }
                if self.reload_timeout.is_some() {
                    ctx.link().send_message(Msg::Load);
                }
            }
            Msg::Dialog(dialog) => {
                if dialog.is_none() && self.dialog.is_some() {
                    ctx.link().send_message(Msg::Load);
                }
                self.dialog = dialog;
            }
            Msg::EditProperty(property) => {
                let url = props.editor_url();
                let dialog = EditDialog::from(property.clone())
                    .edit(property.get_name().is_some())
                    .on_done(ctx.link().callback(|_| Msg::Dialog(None)))
                    .loader(typed_load::<QemuConfig>(url.clone()))
                    .on_submit(self.on_submit.clone())
                    .into();
                self.dialog = Some(dialog);
            }
            Msg::Load => {
                let link = ctx.link().clone();
                let url = props.pending_url();
                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&url, None).await;
                    link.send_message(Msg::LoadResult(result));
                }));
            }
            Msg::LoadResult(result) => {
                self.data = match result {
                    Ok(data) => Some(
                        pve_pending_config_array_to_objects(data).map_err(|err| err.to_string()),
                    ),
                    Err(err) => Some(Err(err.to_string())),
                };
                let link = ctx.link().clone();
                self.reload_timeout = Some(Timeout::new(3000, move || {
                    link.send_message(Msg::Load);
                }));
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content =
            crate::widgets::render_loaded_data(&self.data, |data| self.view_list(ctx, data));

        let menu = Menu::new()
            .with_item({
                MenuItem::new(tr!("Add Hard Disk"))
                    .icon_class("fa fa-hdd-o")
                    .on_select(ctx.link().callback({
                        let property = qemu_disk_property(None, Some(props.node.clone()));
                        move |_| Msg::EditProperty(property.clone())
                    }))
            })
            .with_item({
                MenuItem::new(tr!("Add CD/DVD drive"))
                    .icon_class("fa fa-cdrom")
                    .on_select(ctx.link().callback({
                        let property = qemu_cdrom_property(None, Some(props.node.clone()));
                        move |_| Msg::EditProperty(property.clone())
                    }))
            })
            .with_item({
                MenuItem::new(tr!("Add Network card"))
                    .icon_class("fa fa-exchange")
                    .on_select(ctx.link().callback({
                        let property = qemu_network_property(None, Some(props.node.clone()));
                        move |_| Msg::EditProperty(property.clone())
                    }))
            });

        let menu_button: Html = MenuButton::new("")
            .icon_class("fa fa-bars")
            .class("circle")
            .menu(menu)
            .into();

        crate::widgets::standard_card(tr!("Hardware"), (), menu_button)
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
