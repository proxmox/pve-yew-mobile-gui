use std::rc::Rc;

use anyhow::Error;
use pwt::touch::{Fab, FabSize, SideDialog};
use pwt::widget::form::{Combobox, Field, Form, FormContext};
use serde_json::json;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Button, Column, Container, MiniScroll, Progress, Row};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::{StorageContent, StorageInfo};

use crate::widgets::{label_field, storage_card, StorageContentPanel};

#[derive(Clone, PartialEq, Properties)]
pub struct VmBackupPanel {
    vmid: u32,
    node: AttrValue,
}

impl VmBackupPanel {
    pub fn new(node: impl Into<AttrValue>, vmid: u32) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

pub enum Msg {
    LoadStorage,
    LoadStorageResult(Result<Vec<StorageInfo>, Error>),
    ActiveStorage(String),
    ShowBackupDialog(bool),
}

pub struct PveVmBackupPanel {
    storage_list: Option<Result<Vec<StorageInfo>, String>>,
    load_storage_guard: Option<AsyncAbortGuard>,
    active_storage: Option<String>,
    show_backup_dialog: bool,
    form_context: FormContext,
}

impl PveVmBackupPanel {
    fn create_backup_panel(&self, ctx: &Context<Self>) -> Html {
        let mode_selector = Combobox::new()
            .required(true)
            .with_item("SNAPSHOT")
            .with_item("SUSPEND")
            .with_item("STOP");

        let comp_selector = Combobox::new()
            .required(true)
            .default("zstd")
            .with_item("none")
            .with_item("gzip")
            .with_item("lzo")
            .with_item("zstd")
            .render_value(|comp: &AttrValue| {
                let text = match comp.as_str() {
                    "none" => "none",
                    "gzip" => "GZIP (good)",
                    "lzo" => "LZO (fast)",
                    "zstd" => "ZSTD (fast & good)",
                    _ => panic!("unknown compression mode - internal error"),
                };
                text.into()
            });

        Form::new()
            .form_context(self.form_context.clone())
            .class(pwt::css::FlexFit)
            .with_child(
                Column::new()
                    .class(pwt::css::FlexFit)
                    .padding(2)
                    .gap(2)
                    .with_child(label_field(tr!("Mode"), mode_selector))
                    .with_child(label_field(tr!("Compression"), comp_selector))
                    .with_child(label_field(tr!("Email to"), Field::new()))
                    .with_child(
                        Row::new()
                            .class(pwt::css::JustifyContent::Center)
                            .with_child(
                                Button::new(tr!("Start backup now"))
                                    .icon_class("fa fa-floppy-o")
                                    .class("pwt-button-outline"),
                            ),
                    ),
            )
            .into()
    }

    fn view_config(&self, ctx: &Context<Self>, storage_list: &[StorageInfo]) -> Html {
        let props = ctx.props();

        let mut row = Row::new().gap(2).padding(2);

        if storage_list.is_empty() {
        } else {
            for info in storage_list {
                if !info.enabled.unwrap_or(true) {
                    continue;
                }
                let active = self.active_storage.as_deref() == Some(&info.storage);

                row.add_child(
                    storage_card(
                        &info.storage,
                        info.ty.as_str(),
                        info.shared.unwrap_or(false),
                        &info
                            .content
                            .iter()
                            .map(|c| c.to_string())
                            .collect::<Vec<String>>()
                            .join(", "),
                        info.total,
                        info.used,
                    )
                    .class(active.then(|| pwt::css::ColorScheme::PrimaryContainer))
                    .class(if active {
                        "pwt-elevation4"
                    } else {
                        "pwt-elevation1"
                    })
                    .onclick(ctx.link().callback({
                        let name = info.storage.clone();
                        move |_| Msg::ActiveStorage(name.clone())
                    })),
                )
            }
        }

        let content: Html = if let Some(active_store) = &self.active_storage {
            StorageContentPanel::new(props.node.clone(), active_store.clone())
                .vmid_filter(props.vmid)
                .content_filter(StorageContent::Backup)
                .into()
        } else {
            Column::new()
                .padding(2)
                .class(pwt::css::FlexFit)
                .class(pwt::css::JustifyContent::Center)
                .class(pwt::css::AlignItems::Center)
                .with_child(tr!("Please select a target storage."))
                .into()
        };

        let fab = self.active_storage.is_some().then(|| {
            Fab::new("fa fa-floppy-o")
                .size(FabSize::Small)
                .text("Backup now")
                .class("pwt-position-absolute")
                .style("right", "var(--pwt-spacer-2)")
                .style("bottom", "var(--pwt-spacer-2)")
                .on_activate(ctx.link().callback(|_| Msg::ShowBackupDialog(true)))
        });

        let backup_dialog = self.show_backup_dialog.then(|| {
            SideDialog::new()
                .direction(pwt::touch::SideDialogLocation::Bottom)
                .on_close(ctx.link().callback(|_| Msg::ShowBackupDialog(false)))
                .with_child(self.create_backup_panel(ctx))
        });

        Column::new()
            .class(pwt::css::FlexFit)
            .with_child(MiniScroll::new(row).class(pwt::css::Flex::None))
            .with_child(
                Row::new()
                    .padding_top(1)
                    .padding_x(2)
                    .padding_bottom(2)
                    .with_child(html!{<div class="pwt-font-size-title-medium">{tr!("Recent backups")}</div>}),
            )
            .with_child(content)
            .with_optional_child(fab)
            .with_optional_child(backup_dialog)
            .into()
    }
}

impl Component for PveVmBackupPanel {
    type Message = Msg;
    type Properties = VmBackupPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::LoadStorage);
        Self {
            storage_list: None,
            load_storage_guard: None,
            active_storage: None,
            show_backup_dialog: false,
            form_context: FormContext::new(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.storage_list = None;
        ctx.link().send_message(Msg::LoadStorage);
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ShowBackupDialog(show_backup_dialog) => {
                self.show_backup_dialog = show_backup_dialog;
            }
            Msg::ActiveStorage(name) => {
                self.active_storage = Some(name);
            }
            Msg::LoadStorage => {
                let link = ctx.link().clone();
                let url = format!("/nodes/{}/storage", percent_encode_component(&props.node));
                self.load_storage_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&url, Some(json!({"content": "backup"}))).await;
                    let result = result.map(|mut l: Vec<StorageInfo>| {
                        l.sort_by_key(|i| i.storage.clone());
                        l
                    });
                    link.send_message(Msg::LoadStorageResult(result));
                }));
            }
            Msg::LoadStorageResult(result) => {
                self.storage_list = Some(result.map_err(|err| err.to_string()));
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.storage_list {
            Some(Ok(data)) => self.view_config(ctx, data),
            Some(Err(err)) => pwt::widget::error_message(err).into(),
            None => Progress::new().class("pwt-delay-visibility").into(),
        }
    }
}

impl From<VmBackupPanel> for VNode {
    fn from(props: VmBackupPanel) -> Self {
        let comp = VComp::new::<PveVmBackupPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
