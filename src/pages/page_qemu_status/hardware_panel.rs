use std::collections::HashSet;
use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use proxmox_yew_comp::http_put;
use serde_json::{json, Value};

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::{SnackBar, SnackBarContextExt};
use pwt::widget::menu::{Menu, MenuButton, MenuItem};
use pwt::widget::{Fa, List, ListTile};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};
use pwt::props::{IntoOptionalInlineHtml, SubmitCallback};

use pve_api_types::{PveQmIde, PveQmIdeMedia, QemuConfig};

use crate::api_types::QemuPendingConfigValue;
use crate::form::{
    qemu_bios_property, qemu_cpu_flags_property, qemu_display_property,
    qemu_kernel_scheduler_property, qemu_machine_property, qemu_memory_property,
    qemu_network_mtu_property, qemu_network_property, qemu_scsihw_property,
    qemu_sockets_cores_property, typed_load,
};
use crate::widgets::{
    icon_list_tile, pve_pending_config_array_to_objects, EditDialog, EditableProperty,
    PendingPropertyList,
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

pub fn qemu_pending_config_array_to_rust(
    data: Vec<QemuPendingConfigValue>,
) -> Result<(QemuConfig, QemuConfig, HashSet<String>), Error> {
    let (current, pending, changes) = pve_pending_config_array_to_objects(data)?;
    let current = serde_json::from_value(current)?;
    let pending = serde_json::from_value(pending)?;
    Ok((current, pending, changes))
}

pub enum Msg {
    Load,
    LoadResult(Result<Vec<QemuPendingConfigValue>, Error>),
    Dialog(Option<Html>),
    EditProperty(EditableProperty),
    AddProperty(EditableProperty),
    Revert(EditableProperty),
    RevertResult(Result<(), Error>),
}

pub struct PveQemuHardwarePanel {
    data: Option<Result<(QemuConfig, QemuConfig, HashSet<String>), String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    revert_guard: Option<AsyncAbortGuard>,
    dialog: Option<Html>,

    memory_property: EditableProperty,
    bios_property: EditableProperty,
    sockets_cores_property: EditableProperty,
    cpu_flags_property: EditableProperty,
    kernel_scheduler_property: EditableProperty,
    display_property: EditableProperty,
    machine_property: EditableProperty,
    scsihw_property: EditableProperty,

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
            );

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
        (data, pending, _changes): &(QemuConfig, QemuConfig, HashSet<String>),
    ) -> Html {
        let record: Value = serde_json::to_value(data).unwrap();
        let pending: Value = serde_json::to_value(pending).unwrap();

        let mut list: Vec<ListTile> = Vec::new();

        let push_property_tile = |list: &mut Vec<_>, property, icon| {
            list.push(self.property_tile(ctx, &record, &pending, property, icon, ()));
        };

        push_property_tile(&mut list, self.memory_property.clone(), Fa::new("memory"));
        list.push(self.processor_list_tile(ctx, &record, &pending));
        push_property_tile(&mut list, self.bios_property.clone(), Fa::new("microchip"));
        push_property_tile(&mut list, self.display_property.clone(), Fa::new("desktop"));
        push_property_tile(&mut list, self.machine_property.clone(), Fa::new("cogs"));
        push_property_tile(&mut list, self.scsihw_property.clone(), Fa::new("database"));

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

        for (n, _net_config) in &data.net {
            let name = format!("net{n}");
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
            revert_guard: None,
            dialog: None,
            memory_property: qemu_memory_property(),
            bios_property: qemu_bios_property(),
            sockets_cores_property: qemu_sockets_cores_property(),
            kernel_scheduler_property: qemu_kernel_scheduler_property(),
            cpu_flags_property: qemu_cpu_flags_property(),
            display_property: qemu_display_property(),
            machine_property: qemu_machine_property(),
            scsihw_property: qemu_scsihw_property(),

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
                self.revert_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = on_submit.apply(param).await;
                    link.send_message(Msg::RevertResult(result));
                }));
            }
            Msg::RevertResult(result) => {
                if let Err(err) = result {
                    ctx.link().show_snackbar(
                        SnackBar::new()
                            .message(tr!("Revert property failed") + " - " + &err.to_string()),
                    );
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
                    .on_done(ctx.link().callback(|_| Msg::Dialog(None)))
                    .loader(typed_load::<QemuConfig>(url.clone()))
                    .on_submit(self.on_submit.clone())
                    .into();
                self.dialog = Some(dialog);
            }
            Msg::AddProperty(property) => {
                let url = props.editor_url();
                let dialog = EditDialog::from(property.clone())
                    .edit(false)
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
                    Ok(data) => {
                        Some(qemu_pending_config_array_to_rust(data).map_err(|err| err.to_string()))
                    }
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

        let menu = Menu::new().with_item({
            MenuItem::new(tr!("Add Network card")).on_select(ctx.link().callback({
                let property = qemu_network_property(None, Some(props.node.clone()));
                move |_| Msg::AddProperty(property.clone())
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
