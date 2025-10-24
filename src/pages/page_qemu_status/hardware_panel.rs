use std::rc::Rc;

use anyhow::Error;

use proxmox_schema::property_string::PropertyString;
use proxmox_yew_comp::{http_post, http_put, SafeConfirmDialog};
use serde_json::{json, Value};

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::menu::{Menu, MenuButton, MenuItem};
use pwt::widget::{ConfirmDialog, Container, Fa, List, ListTile};

use proxmox_yew_comp::percent_encoding::percent_encode_component;
use pwt::props::{IntoOptionalInlineHtml, SubmitCallback};

use pve_api_types::{
    PveQmIde, PveQmIdeMedia, QemuConfig, QemuConfigIdeArray, QemuConfigNetArray, QemuConfigSata,
    QemuConfigSataArray, QemuConfigScsi, QemuConfigScsiArray, QemuConfigUnusedArray,
    QemuConfigVirtioArray,
};

use proxmox_yew_comp::form::pve::{
    qemu_bios_property, qemu_cdrom_property, qemu_cpu_flags_property, qemu_disk_property,
    qemu_display_property, qemu_efidisk_property, qemu_kernel_scheduler_property,
    qemu_machine_property, qemu_memory_property, qemu_network_mtu_property, qemu_network_property,
    qemu_scsihw_property, qemu_sockets_cores_property, qemu_tpmstate_property,
    qemu_unused_disk_property, qemu_vmstate_property, typed_load,
};
use proxmox_yew_comp::pending_property_view::{
    pending_typed_load, PendingPropertyList, PendingPropertyView, PendingPropertyViewMsg,
    PendingPropertyViewState, PvePendingConfiguration, PvePendingPropertyView,
};
use proxmox_yew_comp::EditableProperty;

use pwt_macros::builder;

use crate::pages::page_qemu_status::{
    qemu_move_disk_dialog, qemu_reassign_disk_dialog, qemu_resize_disk_dialog,
};
use crate::widgets::standard_card;

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

    pub(crate) fn editor_url(&self) -> String {
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
    CommandResult(Result<(), Error>),
    DeleteDevice(String),
    ResizeDisk(String),
    ReassignDisk(String),
    MoveDisk(String),
}

pub struct PveQemuHardwarePanel {
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

#[derive(Copy, Clone, PartialEq)]
enum EditAction {
    None,
    Edit,
    Add,
}

impl PveQemuHardwarePanel {
    fn property_tile(
        &self,
        ctx: &PveQemuHardwarePanelContext,
        current: &Value,
        pending: &Value,
        property: EditableProperty,
        icon: Fa,
        trailing: impl IntoOptionalInlineHtml,
        edit_action: EditAction,
    ) -> ListTile {
        let on_revert = Callback::from({
            let property = property.clone();
            ctx.link()
                .callback(move |_: Event| PendingPropertyViewMsg::RevertProperty(property.clone()))
        });

        let mut list_tile = PendingPropertyList::render_icon_list_tile(
            current, pending, &property, icon, trailing, on_revert,
        );

        match edit_action {
            EditAction::None => { /* do nothing  */ }
            EditAction::Add | EditAction::Edit => {
                list_tile.set_interactive(true);
                list_tile.set_on_activate(ctx.link().callback({
                    let property = property.clone();
                    move |_| {
                        if edit_action == EditAction::Edit {
                            PendingPropertyViewMsg::EditProperty(property.clone())
                        } else {
                            PendingPropertyViewMsg::AddProperty(property.clone())
                        }
                    }
                }));
            }
        }

        list_tile
    }

    fn processor_list_tile(
        &self,
        ctx: &PveQemuHardwarePanelContext,
        record: &Value,
        pending: &Value,
    ) -> ListTile {
        let menu = Menu::new()
            .with_item(MenuItem::new(&self.sockets_cores_property.title).on_select(
                ctx.link().callback({
                    let property = self.sockets_cores_property.clone();
                    move |_| PendingPropertyViewMsg::EditProperty(property.clone())
                }),
            ))
            .with_item(
                MenuItem::new(&self.kernel_scheduler_property.title).on_select(
                    ctx.link().callback({
                        let property = self.kernel_scheduler_property.clone();
                        move |_| PendingPropertyViewMsg::EditProperty(property.clone())
                    }),
                ),
            )
            .with_item(MenuItem::new(&self.cpu_flags_property.title).on_select(
                ctx.link().callback({
                    let property = self.cpu_flags_property.clone();
                    move |_| PendingPropertyViewMsg::EditProperty(property.clone())
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
            EditAction::Edit,
        );

        tile
    }

    fn network_list_tile(
        &self,
        ctx: &PveQemuHardwarePanelContext,
        name: &str,
        record: &Value,
        pending: &Value,
    ) -> ListTile {
        let props = ctx.props();
        let network_property =
            qemu_network_property(Some(name.to_string()), Some(props.node.clone()));
        let mtu_property =
            qemu_network_mtu_property(Some(name.to_string()), Some(props.node.clone()));

        let menu =
            Menu::new()
                .with_item(
                    MenuItem::new(&network_property.title).on_select(ctx.link().callback({
                        let property = network_property.clone();
                        move |_| PendingPropertyViewMsg::EditProperty(property.clone())
                    })),
                )
                .with_item(
                    MenuItem::new(&mtu_property.title).on_select(ctx.link().callback({
                        let property = mtu_property.clone();
                        move |_| PendingPropertyViewMsg::EditProperty(property.clone())
                    })),
                )
                .with_item({
                    let link = ctx.link().clone();
                    let dialog: Html = SafeConfirmDialog::new(name.to_string())
                        .on_done(link.callback(|_| PendingPropertyViewMsg::ShowDialog(None)))
                        .on_confirm(link.callback({
                            let name = name.to_string();
                            move |_| PendingPropertyViewMsg::Custom(Msg::DeleteDevice(name.clone()))
                        }))
                        .into();
                    MenuItem::new(tr!("Delete device")).on_select(ctx.link().callback(move |_| {
                        PendingPropertyViewMsg::ShowDialog(Some(dialog.clone()))
                    }))
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
            EditAction::Edit,
        );

        tile
    }

    fn move_disk_submit(&self, ctx: &PveQemuHardwarePanelContext) -> SubmitCallback<Value> {
        let props = ctx.props();
        let submit_url = props.move_disk_url();
        let on_start_command = props.on_start_command.clone();

        SubmitCallback::new(move |v: Value| {
            let submit_url = submit_url.clone();
            let on_start_command = on_start_command.clone();
            async move {
                let result: Result<String, Error> = http_post(&submit_url, Some(v)).await;
                if let Some(on_start_command) = &on_start_command {
                    on_start_command.emit(result);
                }
                Ok(())
            }
        })
    }

    fn resize_disk_dialog(&self, ctx: &PveQemuHardwarePanelContext, name: &str) -> Html {
        let props = ctx.props();

        let load_url = props.editor_url();
        let submit_url = props.resize_disk_url();

        qemu_resize_disk_dialog(name, Some(props.node.clone()))
            .on_done(
                ctx.link()
                    .callback(|_| PendingPropertyViewMsg::ShowDialog(None)),
            )
            .loader(typed_load::<QemuConfig>(load_url.clone()))
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
            .into()
    }

    fn disk_list_tile(
        &self,
        ctx: &PveQemuHardwarePanelContext,
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
                MenuItem::new(tr!("Move Disk")).on_select(
                    ctx.link().callback(move |_| {
                        PendingPropertyViewMsg::Custom(Msg::MoveDisk(name.clone()))
                    }),
                )
            });
            menu.add_item({
                let name = name.to_string();
                MenuItem::new(tr!("Reassign Disk")).on_select(ctx.link().callback(move |_| {
                    PendingPropertyViewMsg::Custom(Msg::ReassignDisk(name.clone()))
                }))
            });
            menu.add_item({
                let name = name.to_string();
                MenuItem::new(tr!("Resize Disk")).on_select(ctx.link().callback(move |_| {
                    PendingPropertyViewMsg::Custom(Msg::ResizeDisk(name.clone()))
                }))
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
                .on_done(link.callback(|_| PendingPropertyViewMsg::ShowDialog(None)))
                .on_confirm(link.callback({
                    let name = name.to_string();
                    move |_| PendingPropertyViewMsg::Custom(Msg::DeleteDevice(name.clone()))
                }))
                .into();
            MenuItem::new(title).on_select(
                ctx.link()
                    .callback(move |_| PendingPropertyViewMsg::ShowDialog(Some(dialog.clone()))),
            )
        });

        let menu_button: Html = MenuButton::new("")
            .class(pwt::css::ColorScheme::Neutral)
            .class("circle")
            .icon_class("fa fa-ellipsis-v fa-lg")
            .menu(menu)
            .into();
        let mut tile = self.property_tile(
            ctx,
            &record,
            &pending,
            property,
            icon,
            menu_button,
            EditAction::Edit,
        );
        tile.set_key(name.to_string());
        tile
    }

    fn unused_disk_list_tile(
        &self,
        ctx: &PveQemuHardwarePanelContext,
        name: &str,
        record: &Value,
        pending: &Value,
    ) -> ListTile {
        let props = ctx.props();

        let menu = Menu::new().with_item({
            let link = ctx.link().clone();

            let dialog: Html = ConfirmDialog::default()
                .on_close(link.callback(|_| PendingPropertyViewMsg::ShowDialog(None)))
                .on_confirm(link.callback({
                    let name = name.to_string();
                    move |_| PendingPropertyViewMsg::Custom(Msg::DeleteDevice(name.clone()))
                }))
                .into();

            MenuItem::new(tr!("Delete disk")).on_select(
                ctx.link()
                    .callback(move |_| PendingPropertyViewMsg::ShowDialog(Some(dialog.clone()))),
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
        let mut tile = self.property_tile(
            ctx,
            &record,
            &pending,
            property,
            icon,
            menu_button,
            EditAction::Add,
        );
        tile.set_key(name.to_string());
        tile
    }

    fn view_list(&self, ctx: &PveQemuHardwarePanelContext, data: &PvePendingConfiguration) -> Html {
        let props = ctx.props();
        let mut list: Vec<ListTile> = Vec::new();

        let PvePendingConfiguration {
            current,
            pending,
            keys,
        } = data;

        let push_property_tile = |list: &mut Vec<_>, property: EditableProperty, icon, editable| {
            let name = match property.get_name() {
                Some(name) => name.to_string(),
                None::<_> => return,
            };

            if property.required || keys.contains(&name) {
                let mut tile =
                    self.property_tile(ctx, current, pending, property, icon, (), editable);
                tile.set_key(name);
                list.push(tile);
            }
        };

        push_property_tile(
            &mut list,
            self.memory_property.clone(),
            Fa::new("memory"),
            EditAction::Edit,
        );
        list.push(self.processor_list_tile(ctx, current, pending));
        push_property_tile(
            &mut list,
            self.bios_property.clone(),
            Fa::new("microchip"),
            EditAction::Edit,
        );
        push_property_tile(
            &mut list,
            self.display_property.clone(),
            Fa::new("desktop"),
            EditAction::Edit,
        );
        push_property_tile(
            &mut list,
            self.machine_property.clone(),
            Fa::new("cogs"),
            EditAction::Edit,
        );
        push_property_tile(
            &mut list,
            self.scsihw_property.clone(),
            Fa::new("database"),
            EditAction::Edit,
        );

        // fixme: this should be removable - add menu with delete
        push_property_tile(
            &mut list,
            self.vmstate_property.clone(),
            Fa::new("download"),
            EditAction::Edit,
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
                Ok(None::<_>) => PveQmIdeMedia::Disk,
                Err(err) => {
                    log::error!("unable to parse drive '{name}' media: {err}");
                    continue;
                }
            };
            list.push(self.disk_list_tile(ctx, &name, media, current, pending));
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
                Ok(None::<_>) => PveQmIdeMedia::Disk,
                Err(err) => {
                    log::error!("unable to parse drive '{name}' media: {err}");
                    continue;
                }
            };
            list.push(self.disk_list_tile(ctx, &name, media, current, pending));
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
                Ok(None::<_>) => PveQmIdeMedia::Disk,
                Err(err) => {
                    log::error!("unable to parse drive '{name}' media: {err}");
                    continue;
                }
            };
            list.push(self.disk_list_tile(ctx, &name, media, current, pending));
        }

        for n in 0..QemuConfigVirtioArray::MAX {
            let name = format!("virtio{n}");
            if !keys.contains(&name) {
                continue;
            }
            list.push(self.disk_list_tile(ctx, &name, PveQmIdeMedia::Disk, current, pending));
        }

        for n in 0..QemuConfigNetArray::MAX {
            let name = format!("net{n}");
            if !keys.contains(&name) {
                continue;
            }
            list.push(self.network_list_tile(ctx, &name, current, pending));
        }

        for n in 0..QemuConfigUnusedArray::MAX {
            let name = format!("unused{n}");
            if !keys.contains(&name) {
                continue;
            }
            list.push(self.unused_disk_list_tile(ctx, &name, current, pending));
        }

        let property = qemu_efidisk_property(Some("edidisk0".into()), Some(props.node.clone()));
        push_property_tile(&mut list, property, Fa::new("hdd-o"), EditAction::None);

        let property = qemu_tpmstate_property(Some("tpmstate0".into()), Some(props.node.clone()));
        push_property_tile(&mut list, property, Fa::new("hdd-o"), EditAction::None);

        List::from_tiles(list)
            .grid_template_columns("auto 1fr")
            .into()
    }

    fn card_menu(&self, ctx: &PveQemuHardwarePanelContext, data: &PvePendingConfiguration) -> Html {
        let props = ctx.props();

        let PvePendingConfiguration {
            current: _,
            pending,
            keys: _,
        } = data;

        let has_efidisk = pending.get("efidisk0").is_some();
        let has_tpmstate = pending.get("tpmstate0").is_some();

        let menu = Menu::new()
            .with_item({
                MenuItem::new(tr!("Add Hard Disk"))
                    .icon_class("fa fa-hdd-o")
                    .on_select(ctx.link().callback({
                        let property = qemu_disk_property(None, Some(props.node.clone()));
                        move |_| PendingPropertyViewMsg::AddProperty(property.clone())
                    }))
            })
            .with_item({
                MenuItem::new(tr!("Add CD/DVD drive"))
                    .icon_class("fa fa-cdrom")
                    .on_select(ctx.link().callback({
                        let property = qemu_cdrom_property(None, Some(props.node.clone()));
                        move |_| PendingPropertyViewMsg::AddProperty(property.clone())
                    }))
            })
            .with_item({
                MenuItem::new(tr!("Add Network card"))
                    .icon_class("fa fa-exchange")
                    .on_select(ctx.link().callback({
                        let property = qemu_network_property(None, Some(props.node.clone()));
                        move |_| PendingPropertyViewMsg::AddProperty(property.clone())
                    }))
            })
            .with_item({
                MenuItem::new(tr!("EFI Disk"))
                    .icon_class("fa fa-hdd-o")
                    .disabled(has_efidisk)
                    .on_select(ctx.link().callback({
                        let property = qemu_efidisk_property(None, Some(props.node.clone()));
                        move |_| PendingPropertyViewMsg::AddProperty(property.clone())
                    }))
            })
            .with_item({
                MenuItem::new(tr!("TPM State"))
                    .icon_class("fa fa-hdd-o")
                    .disabled(has_tpmstate)
                    .on_select(ctx.link().callback({
                        let property = qemu_tpmstate_property(None, Some(props.node.clone()));
                        move |_| PendingPropertyViewMsg::AddProperty(property.clone())
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

type PveQemuHardwarePanelContext = Context<PvePendingPropertyView<PveQemuHardwarePanel>>;

impl PendingPropertyView for PveQemuHardwarePanel {
    type Message = Msg;
    type Properties = QemuHardwarePanel;
    const MOBILE: bool = true;

    fn create(ctx: &PveQemuHardwarePanelContext) -> Self {
        Self {
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

    fn changed(
        &mut self,
        ctx: &PveQemuHardwarePanelContext,
        _view_state: &mut PendingPropertyViewState,
        old_props: &Self::Properties,
    ) -> bool {
        let props = ctx.props();

        if props.node != old_props.node || props.vmid != old_props.vmid {
            self.on_submit = Self::create_on_submit(ctx.props());
            ctx.link().send_message(PendingPropertyViewMsg::Load);
        }
        true
    }

    fn update(
        &mut self,
        ctx: &PveQemuHardwarePanelContext,
        view_state: &mut PendingPropertyViewState,
        msg: Self::Message,
    ) -> bool {
        let props = ctx.props();

        match msg {
            Msg::DeleteDevice(name) => {
                let link = ctx.link().clone();
                let on_submit = self.on_submit.clone();
                let param = json!({ "delete": [name] });
                view_state.async_pool.spawn(async move {
                    let result = on_submit.apply(param).await;
                    link.send_message(PendingPropertyViewMsg::Custom(Msg::CommandResult(result)));
                });
            }
            Msg::CommandResult(result) => {
                if let Err(err) = result {
                    crate::show_failed_command_error(ctx.link(), err);
                }
                if view_state.reload_timeout.is_some() {
                    ctx.link().send_message(PendingPropertyViewMsg::Load);
                }
            }
            Msg::ResizeDisk(name) => {
                let dialog = self.resize_disk_dialog(ctx, &name);
                view_state.dialog = Some(dialog);
            }
            Msg::ReassignDisk(name) => {
                let dialog = qemu_reassign_disk_dialog(&name, Some(props.node.clone()))
                    .on_done(
                        ctx.link()
                            .callback(|_| PendingPropertyViewMsg::ShowDialog(None)),
                    )
                    .loader(typed_load::<QemuConfig>(props.editor_url()))
                    .on_submit(self.move_disk_submit(ctx));
                view_state.dialog = Some(dialog.into());
            }
            Msg::MoveDisk(name) => {
                let dialog = qemu_move_disk_dialog(&name, Some(props.node.clone()))
                    .on_done(
                        ctx.link()
                            .callback(|_| PendingPropertyViewMsg::ShowDialog(None)),
                    )
                    .loader(typed_load::<QemuConfig>(props.editor_url()))
                    .on_submit(self.move_disk_submit(ctx));
                view_state.dialog = Some(dialog.into());
            }
        }
        true
    }

    fn view(
        &self,
        ctx: &PveQemuHardwarePanelContext,
        view_state: &PendingPropertyViewState,
    ) -> Html {
        let title = tr!("Hardware");
        let min_height = 200;

        let PendingPropertyViewState {
            data,
            error,
            dialog,
            ..
        } = view_state;

        let card = match (data, &error) {
            (None, None) => standard_card(title, (), ())
                .class(pwt::css::Display::Flex)
                .class(pwt::css::FlexDirection::Column)
                .min_height(min_height)
                .with_child(pwt::widget::Progress::new().class("pwt-delay-visibility"))
                .with_child(
                    Container::new()
                        .class(pwt::css::FlexFit)
                        .class("pwt-bg-color-neutral"),
                ),
            (None, Some(err)) => standard_card(title, (), ())
                .class(pwt::css::Display::Flex)
                .class(pwt::css::FlexDirection::Column)
                .min_height(min_height)
                .with_child(
                    pwt::widget::error_message(&err.to_string())
                        .padding(2)
                        .class(pwt::css::FlexFit)
                        .class("pwt-bg-color-neutral"),
                ),
            (Some(data), Some(err)) => {
                let card_menu = self.card_menu(ctx, data);
                standard_card(title, (), card_menu)
                    .with_child(
                        pwt::widget::error_message(&err.to_string())
                            .padding(2)
                            .border_bottom(true)
                            .class("pwt-bg-color-neutral"),
                    )
                    .with_child(self.view_list(ctx, data))
            }
            (Some(data), None::<_>) => {
                let card_menu = self.card_menu(ctx, data);
                standard_card(title, (), card_menu).with_child(self.view_list(ctx, data))
            }
        };
        card.with_optional_child(dialog.clone()).into()
    }

    fn editor_loader(props: &Self::Properties) -> Option<proxmox_yew_comp::ApiLoadCallback<Value>> {
        let url = props.editor_url();
        Some(typed_load::<QemuConfig>(url.clone()))
    }

    fn pending_loader(
        props: &Self::Properties,
    ) -> Option<proxmox_yew_comp::ApiLoadCallback<PvePendingConfiguration>> {
        let pending_url = props.pending_url();
        Some(pending_typed_load::<QemuConfig>(pending_url.clone()))
    }

    fn on_submit(props: &Self::Properties) -> Option<SubmitCallback<Value>> {
        Some(Self::create_on_submit(props))
    }
}

impl From<QemuHardwarePanel> for VNode {
    fn from(props: QemuHardwarePanel) -> Self {
        let comp = VComp::new::<PvePendingPropertyView<PveQemuHardwarePanel>>(Rc::new(props), None);
        VNode::from(comp)
    }
}
