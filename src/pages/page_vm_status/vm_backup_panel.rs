use std::rc::Rc;

use anyhow::Error;
use proxmox_human_byte::HumanByte;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Card, Column, Fa, MiniScroll, Progress, Row};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::{StorageContent, StorageInfo};

use crate::widgets::StorageContentPanel;

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
}

pub struct PveVmBackupPanel {
    storage_list: Option<Result<Vec<StorageInfo>, String>>,
    load_storage_guard: Option<AsyncAbortGuard>,
    active_storage: Option<String>,
}

pub fn storage_card(info: &StorageInfo) -> Card {
    let (usage_text, percentage) = if let (Some(total), Some(used)) = (info.total, info.used) {
        let left_text = HumanByte::new_binary(used as f64);
        let right_text = HumanByte::new_binary(total as f64);

        (Row::new()
            .gap(2)
            .class("pwt-align-items-flex-end")
            .with_child(html! {
                <div class="pwt-font-size-title-small pwt-flex-fill">{left_text.to_string()}</div>
            })
            .with_flex_spacer()
            .with_child(html! {
                <div class="pwt-font-size-title-small">{right_text.to_string()}</div>
            }), (used as f32) / (total as f32))
    } else {
        (
            Row::new()
                .gap(2)
                .with_child(tr!("Storage size/usage unknown")),
            0.0,
        )
    };

    let usage = Column::new()
        .gap(1)
        .with_child(usage_text)
        .with_child(Progress::new().value(percentage));

    let content_text = Column::new()
        .gap(1)
        .class(pwt::css::Flex::Fill)
        .class(pwt::css::AlignItems::Center)
        .with_child(html! {<div class="pwt-font-size-title-medium">{&info.storage}</div>})
        .with_child(
            html! {<div class="pwt-font-size-title-small">{&format!("({})", info.ty)}</div>},
        );

    let type_icon = match info.ty.as_str() {
        "pbs" => "cloud",
        _ => "folder",
    };

    let content = Row::new()
        .gap(2)
        .with_child(Fa::new(type_icon).large_2x())
        .with_child(content_text);

    Card::new()
        .min_width(250)
        .class("pwt-interactive")
        .with_child(Column::new().gap(1).with_child(content).with_child(usage))
}

impl PveVmBackupPanel {
    fn view_config(&self, ctx: &Context<Self>, storage_list: &[StorageInfo]) -> Html {
        let props = ctx.props();

        let mut row = Row::new().gap(2).padding(2);

        if storage_list.is_empty() {
        } else {
            for info in storage_list {
                let active = self.active_storage.as_deref() == Some(&info.storage);

                row.add_child(
                    storage_card(info)
                        .class(active.then(|| pwt::css::ColorScheme::PrimaryContainer))
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

        Column::new()
            .class(pwt::css::FlexFit)
            .with_child(MiniScroll::new(row).class(pwt::css::Flex::None))
            .with_child(
                Row::new()
                    .border_bottom(true)
                    .padding_top(1)
                    .padding_x(2)
                    .padding_bottom(2)
                    .with_child(html!{<div class="pwt-font-size-title-medium">{tr!("Recent backups")}</div>}),
            )
            .with_child(content)
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
            Msg::ActiveStorage(name) => {
                self.active_storage = Some(name);
            }
            Msg::LoadStorage => {
                let link = ctx.link().clone();
                let url = format!("/nodes/{}/storage", percent_encode_component(&props.node));
                self.load_storage_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&url, None).await;
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
