use std::collections::HashSet;
use std::rc::Rc;

use anyhow::{bail, Error};

use gloo_timers::callback::Timeout;
use proxmox_schema::property_string::PropertyString;
use proxmox_yew_comp::{http_post, http_put, SafeConfirmDialog};
use pwt::widget::form::Number;
use serde_json::{json, Value};

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::widget::menu::{Menu, MenuButton, MenuItem};
use pwt::widget::{Column, ConfirmDialog, Fa, List, ListTile};
use pwt::AsyncAbortGuard;
use pwt::{prelude::*, AsyncPool};

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};
use pwt::props::{IntoOptionalInlineHtml, SubmitCallback};

use pve_api_types::{
    PveQmIde, PveQmIdeMedia, QemuConfig, QemuConfigIdeArray, QemuConfigNetArray, QemuConfigSata,
    QemuConfigSataArray, QemuConfigScsi, QemuConfigScsiArray, QemuConfigUnusedArray,
    QemuConfigVirtioArray,
};

use crate::api_types::QemuPendingConfigValue;
use crate::form::{
    qemu_bios_property, qemu_cdrom_property, qemu_cpu_flags_property, qemu_disk_property,
    qemu_display_property, qemu_efidisk_property, qemu_kernel_scheduler_property,
    qemu_machine_property, qemu_memory_property, qemu_network_mtu_property, qemu_network_property,
    qemu_scsihw_property, qemu_sockets_cores_property, qemu_tpmstate_property,
    qemu_unused_disk_property, qemu_vmstate_property, typed_load,
};
use crate::pages::page_qemu_status::qemu_move_disk_dialog;
use crate::widgets::{
    label_field, pve_pending_config_array_to_objects, standard_card, EditDialog, EditableProperty,
    PendingPropertyList, PropertyEditorState,
};

use pwt_macros::builder;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct QemuHardwarePanel {
    vmid: u32,
    node: AttrValue,

    #[builder_cb(IntoEventCallback, into_event_callback, Result<String, Error>)]
    #[prop_or_default]
    on_start_command: Option<Callback<Result<String, Error>>>,
}

impl QemuHardwarePanel {
    pub fn new(node: impl Into<AttrValue>, vmid: u32) -> Self {
        yew::props!(Self {
            node: node.into(),
            vmid,
        })
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

    fn resize_disk_url(&self) -> String {
        format!(
            "/nodes/{}/qemu/{}/resize",
            percent_encode_component(&self.node),
            self.vmid
        )
    }

    fn move_disk_url(&self) -> String {
        format!(
            "/nodes/{}/qemu/{}/move_disk",
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
    ResizeDisk(String),
    MoveDisk(String),
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
        interactive: bool,
    ) -> ListTile {
        let on_revert = Callback::from({
            let property = property.clone();
            ctx.link()
                .callback(move |_: Event| Msg::Revert(property.clone()))
        });

        let mut list_tile = PendingPropertyList::render_icon_list_tile(
            current, pending, &property, icon, trailing, on_revert,
        );

        if interactive {
            list_tile.set_interactive(true);
            list_tile.set_on_activate(ctx.link().callback({
                let property = property.clone();
                move |_| Msg::EditProperty(property.clone())
            }));
        }

        list_tile
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
            true,
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
            true,
        );

        tile
    }

    fn move_disk_dialog(&self, ctx: &Context<Self>, name: &str) -> Html {
        let props = ctx.props();

        let load_url = props.editor_url();
        let submit_url = props.move_disk_url();

        qemu_move_disk_dialog(name, Some(props.node.clone()))
            .on_done(ctx.link().callback(|_| Msg::Dialog(None)))
            .loader(typed_load::<QemuConfig>(load_url.clone()))
            .on_submit({
                let on_start_command = props.on_start_command.clone();
                move |v: Value| {
                    let submit_url = submit_url.clone();
                    let on_start_command = on_start_command.clone();
                    async move {
                        let result: Result<String, Error> = http_post(&submit_url, Some(v)).await;
                        if let Some(on_start_command) = &on_start_command {
                            on_start_command.emit(result);
                        }
                        Ok(())
                    }
                }
            })
            .into()
    }

    fn resize_disk_dialog(&self, ctx: &Context<Self>, name: &str) -> Html {
        let props = ctx.props();

        let load_url = props.editor_url();
        let submit_url = props.resize_disk_url();
        let title = tr!("Resize Disk");

        EditDialog::new(title.clone() + " (" + name + ")")
            .edit(false)
            .on_done(ctx.link().callback(|_| Msg::Dialog(None)))
            .loader(typed_load::<QemuConfig>(load_url.clone()))
            .submit_text(title.clone())
            .submit_hook({
                let disk = name.to_string();
                move |state: PropertyEditorState| {
                    let mut data = state.form_ctx.get_submit_data(); // get digest
                    let incr = match state
                        .form_ctx
                        .read()
                        .get_last_valid_value("_size_increment_")
                    {
                        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
                        _ => bail!("invalid size increase - internal error"),
                    };
                    data["disk"] = disk.clone().into();
                    data["size"] = format!("+{incr}G").into();
                    Ok(data)
                }
            })
            .on_submit({
                let on_start_command = props.on_start_command.clone();
                move |v: Value| {
                    let submit_url = submit_url.clone();
                    let on_start_command = on_start_command.clone();
                    async move {
                        let result: Result<String, Error> = http_put(&submit_url, Some(v)).await;
                        if let Some(on_start_command) = &on_start_command {
                            on_start_command.emit(result);
                        }
                        Ok(())
                    }
                }
            })
            .renderer(|_| {
                Column::new()
                    .class(pwt::css::FlexFit)
                    .gap(2)
                    .with_child(label_field(
                        tr!("Size Increment") + " (" + &tr!("GiB") + ")",
                        Number::<f64>::new()
                            .name("_size_increment_")
                            .default(0.0)
                            .min(0.0)
                            .max(128.0 * 1024.0)
                            .submit(false),
                        true,
                    ))
                    .into()
            })
            .into()
    }

    fn disk_list_tile(
        &self,
        ctx: &Context<Self>,
        name: &str,
        media: PveQmIdeMedia,
        record: &Value,
        pending: &Value,
    ) -> ListTile {
        let props = ctx.props();

        let (property, icon) = if media == PveQmIdeMedia::Cdrom {
            (
                qemu_cdrom_property(Some(name.to_string()), Some(props.node.clone())),
                Fa::new("cdrom"),
            )
        } else {
            (
                qemu_disk_property(Some(name.to_string()), Some(props.node.clone())),
                Fa::new("hdd-o"),
            )
        };

        let mut menu = Menu::new();
        if media == PveQmIdeMedia::Disk {
            menu.add_item({
                let name = name.to_string();
                MenuItem::new(tr!("Move Disk"))
                    .on_select(ctx.link().callback(move |_| Msg::MoveDisk(name.clone())))
            });
            menu.add_item({
                let name = name.to_string();
                MenuItem::new(tr!("Resize Disk"))
                    .on_select(ctx.link().callback(move |_| Msg::ResizeDisk(name.clone())))
            })
        };

        menu.add_item({
            let link = ctx.link().clone();
            let (title, message) = if media == PveQmIdeMedia::Disk {
                (
                    tr!("Detach disk"),
                    Some(tr!("Are you sure you want to detach entry {0}", name)),
                )
            } else {
                (tr!("Delete device"), None)
            };
            let dialog: Html = SafeConfirmDialog::new(name.to_string())
                .message(message)
                .on_done(link.callback(|_| Msg::Dialog(None)))
                .on_confirm(link.callback({
                    let name = name.to_string();
                    move |_| Msg::DeleteDevice(name.clone())
                }))
                .into();
            MenuItem::new(title).on_select(
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
        let mut tile =
            self.property_tile(ctx, &record, &pending, property, icon, menu_button, true);
        tile.set_key(name.to_string());
        tile
    }

    fn unused_disk_list_tile(
        &self,
        ctx: &Context<Self>,
        name: &str,
        record: &Value,
        pending: &Value,
    ) -> ListTile {
        let props = ctx.props();

        let menu = Menu::new().with_item({
            let link = ctx.link().clone();

            let dialog: Html = ConfirmDialog::default()
                .on_close(link.callback(|_| Msg::Dialog(None)))
                .on_confirm(link.callback({
                    let name = name.to_string();
                    move |_| Msg::DeleteDevice(name.clone())
                }))
                .into();

            MenuItem::new(tr!("Delete disk")).on_select(
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

        let icon = Fa::new("hdd-o");
        let property = qemu_unused_disk_property(&name, Some(props.node.clone()));
        let mut tile =
            self.property_tile(ctx, &record, &pending, property, icon, menu_button, true);
        tile.set_key(name.to_string());
        tile
    }

    fn view_list(
        &self,
        ctx: &Context<Self>,
        (record, pending, keys): &(Value, Value, HashSet<String>),
    ) -> Html {
        let props = ctx.props();
        let mut list: Vec<ListTile> = Vec::new();

        let push_property_tile = |list: &mut Vec<_>, property: EditableProperty, icon, editable| {
            let name = match property.get_name() {
                Some(name) => name.to_string(),
                None::<_> => return,
            };

            if property.required || keys.contains(&name) {
                let mut tile =
                    self.property_tile(ctx, &record, &pending, property, icon, (), editable);
                tile.set_key(name);
                list.push(tile);
            }
        };

        push_property_tile(
            &mut list,
            self.memory_property.clone(),
            Fa::new("memory"),
            true,
        );
        list.push(self.processor_list_tile(ctx, &record, &pending));
        push_property_tile(
            &mut list,
            self.bios_property.clone(),
            Fa::new("microchip"),
            true,
        );
        push_property_tile(
            &mut list,
            self.display_property.clone(),
            Fa::new("desktop"),
            true,
        );
        push_property_tile(
            &mut list,
            self.machine_property.clone(),
            Fa::new("cogs"),
            true,
        );
        push_property_tile(
            &mut list,
            self.scsihw_property.clone(),
            Fa::new("database"),
            true,
        );

        // fixme: this should be removable - add menu with delete
        push_property_tile(
            &mut list,
            self.vmstate_property.clone(),
            Fa::new("download"),
            true,
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
            list.push(self.disk_list_tile(ctx, &name, media, &record, &pending));
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
            list.push(self.disk_list_tile(ctx, &name, media, &record, &pending));
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
            list.push(self.disk_list_tile(ctx, &name, media, &record, &pending));
        }

        for n in 0..QemuConfigVirtioArray::MAX {
            let name = format!("virtio{n}");
            if !keys.contains(&name) {
                continue;
            }
            list.push(self.disk_list_tile(ctx, &name, PveQmIdeMedia::Disk, &record, &pending));
        }

        for n in 0..QemuConfigNetArray::MAX {
            let name = format!("net{n}");
            if !keys.contains(&name) {
                continue;
            }
            list.push(self.network_list_tile(ctx, &name, &record, &pending));
        }

        for n in 0..QemuConfigUnusedArray::MAX {
            let name = format!("unused{n}");
            if !keys.contains(&name) {
                continue;
            }
            list.push(self.unused_disk_list_tile(ctx, &name, record, pending));
        }

        let property = qemu_efidisk_property(Some("edidisk0".into()), Some(props.node.clone()));
        push_property_tile(&mut list, property, Fa::new("hdd-o"), false);

        let property = qemu_tpmstate_property(Some("tpmstate0".into()), Some(props.node.clone()));
        push_property_tile(&mut list, property, Fa::new("hdd-o"), false);

        List::from_tiles(list)
            .grid_template_columns("auto 1fr")
            .into()
    }

    fn card_menu(
        &self,
        ctx: &Context<Self>,
        (_record, pending, _keys): &(Value, Value, HashSet<String>),
    ) -> Html {
        let props = ctx.props();

        let has_efidisk = pending.get("efidisk0").is_some();
        let has_tpmstate = pending.get("tpmstate0").is_some();

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
            })
            .with_item({
                MenuItem::new(tr!("EFI Disk"))
                    .icon_class("fa fa-hdd-o")
                    .disabled(has_efidisk)
                    .on_select(ctx.link().callback({
                        let property = qemu_efidisk_property(None, Some(props.node.clone()));
                        move |_| Msg::EditProperty(property.clone())
                    }))
            })
            .with_item({
                MenuItem::new(tr!("TPM State"))
                    .icon_class("fa fa-hdd-o")
                    .disabled(has_tpmstate)
                    .on_select(ctx.link().callback({
                        let property = qemu_tpmstate_property(None, Some(props.node.clone()));
                        move |_| Msg::EditProperty(property.clone())
                    }))
            });

        MenuButton::new("")
            .icon_class("fa fa-bars")
            .class("circle")
            .menu(menu)
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

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if props.node != old_props.node || props.vmid != old_props.vmid {
            self.data = None;
            self.on_submit = Self::create_on_submit(ctx.props());
            ctx.link().send_message(Msg::Load);
        }
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
            Msg::ResizeDisk(name) => {
                self.dialog = Some(self.resize_disk_dialog(ctx, &name));
            }
            Msg::MoveDisk(name) => {
                self.dialog = Some(self.move_disk_dialog(ctx, &name));
            }
            Msg::Dialog(dialog) => {
                if dialog.is_none() && self.dialog.is_some() {
                    ctx.link().send_message(Msg::Load);
                }
                self.dialog = dialog;
            }
            Msg::EditProperty(property) => {
                let url = props.editor_url();
                let is_edit = if let Some(name) = property.get_name() {
                    if name.starts_with("unused") {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                };
                let dialog = EditDialog::from(property.clone())
                    .edit(is_edit)
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
        let title = tr!("Hardware");
        let min_height = 200;

        let data = match &self.data {
            None => {
                return standard_card(title.clone(), (), ())
                    .min_height(min_height)
                    .with_child(pwt::widget::Progress::new().class("pwt-delay-visibility"))
                    .into()
            }
            Some(Err(err)) => {
                return standard_card(title.clone(), (), ())
                    .min_height(min_height)
                    .with_child(pwt::widget::error_message(&err.to_string()).padding(2))
                    .into();
            }
            Some(Ok(data)) => data,
        };

        let content = self.view_list(ctx, data);
        let card_menu = self.card_menu(ctx, data);

        crate::widgets::standard_card(title, (), card_menu)
            .min_height(min_height)
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
