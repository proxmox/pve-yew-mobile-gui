use std::rc::Rc;

use anyhow::Error;
use proxmox_human_byte::HumanByte;
use pwt::widget::form::Field;
use serde_json::json;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::MaterialAppScopeExt;
use pwt::widget::{Column, Container, Dialog, Fa, List, ListTile, Row, Trigger};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::layout::list_tile::icon_list_tile;
use proxmox_yew_comp::layout::render_loaded_data;

use proxmox_yew_comp::{http_delete_get, http_get, percent_encoding::percent_encode_component};

use pve_api_types::StorageContent;

use crate::widgets::VolumeActionDialog;
use proxmox_yew_comp::pve_api_types::StorageEntry;

#[derive(Clone, PartialEq, Properties)]
pub struct StorageContentPanel {
    storage: AttrValue,
    node: AttrValue,

    #[prop_or_default]
    content_filter: Option<StorageContent>,

    #[prop_or_default]
    vmid_filter: Option<u32>,
}

impl StorageContentPanel {
    pub fn new(node: impl Into<AttrValue>, storage: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            node: node.into(),
            storage: storage.into(),
        })
    }

    pub fn content_filter(mut self, filter: impl IntoPropValue<Option<StorageContent>>) -> Self {
        self.set_content_filter(filter);
        self
    }

    pub fn set_content_filter(&mut self, filter: impl IntoPropValue<Option<StorageContent>>) {
        self.content_filter = filter.into_prop_value();
    }

    pub fn vmid_filter(mut self, vmid: impl IntoPropValue<Option<u32>>) -> Self {
        self.set_vmid_filter(vmid);
        self
    }

    pub fn set_vmid_filter(&mut self, vmid: impl IntoPropValue<Option<u32>>) {
        self.vmid_filter = vmid.into_prop_value();
    }
}
pub enum Msg {
    Load,
    LoadResult(Result<Vec<StorageEntry>, Error>),
    SetFilter(String),
    ShowContentDialog(StorageEntry),
    Remove(String),
    RemoveResult(Result<String, Error>),
    ExtractConfig(String),
    ExtractConfigResult(Result<String, Error>),
    CloseConfigDialog,
}

pub struct PveStorageContentPanel {
    filter: String,
    data: Option<Result<Vec<StorageEntry>, String>>,
    load_guard: Option<AsyncAbortGuard>,
    remove_guard: Option<AsyncAbortGuard>,
    extract_config_guard: Option<AsyncAbortGuard>,
    config_dialog: Option<Html>,
}

fn get_content_icon(content: &str) -> &str {
    match content {
        "iso" => "cdrom",
        "vztmpl" | "rootdir" => "cube",
        "images" => "hdd-o",
        "backup" => "floppy-o",
        _ => "file-o",
    }
}

impl PveStorageContentPanel {
    fn view_list(&self, ctx: &Context<Self>, data: &[StorageEntry]) -> Html {
        let mut list: Vec<ListTile> = Vec::new();

        for item in data {
            let item = item.clone();
            list.push(
                icon_list_tile(
                    Fa::new(get_content_icon(&item.content)).class("pwt-color-secondary"),
                    item.volid.clone(),
                    format!("Size {}", HumanByte::new_binary(item.size as f64)),
                    Html::from(Fa::new("ellipsis-v").large()),
                )
                .interactive(true)
                .onclick(
                    ctx.link()
                        .callback(move |_| Msg::ShowContentDialog(item.clone())),
                ),
            );
        }

        List::from_tiles(list)
            .class(pwt::css::FlexFit)
            .grid_template_columns("auto 1fr auto")
            .border_top(true)
            .into()
    }

    fn view_content(&self, ctx: &Context<Self>, data: &[StorageEntry]) -> Html {
        let search = Row::new().padding_x(2).padding_bottom(2).with_child({
            let mut field = Field::new()
                .value(self.filter.clone())
                .on_input(ctx.link().callback(Msg::SetFilter));
            if !self.filter.is_empty() {
                field.add_trigger(
                    Trigger::new("fa fa-times")
                        .on_activate(ctx.link().callback(|_| Msg::SetFilter(String::new()))),
                    true,
                );
            }

            field.add_trigger(Trigger::new("fa fa-search pwt-opacity-50"), true);

            field
        });

        let data: Vec<StorageEntry> = if self.filter.is_empty() {
            data.to_vec()
        } else {
            data.iter()
                .filter(|item| item.volid.contains(&self.filter))
                .cloned()
                .collect()
        };

        Column::new()
            .class(pwt::css::FlexFit)
            .with_child(search)
            .with_child(self.view_list(ctx, &data))
            .with_optional_child(self.config_dialog.clone())
            .into()
    }
}

impl Component for PveStorageContentPanel {
    type Message = Msg;
    type Properties = StorageContentPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
            load_guard: None,
            remove_guard: None,
            extract_config_guard: None,
            filter: String::new(),
            config_dialog: None,
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
            Msg::SetFilter(filter) => {
                self.filter = filter;
            }
            Msg::Load => {
                let link = ctx.link().clone();
                let url = format!(
                    "/nodes/{}/storage/{}/content",
                    percent_encode_component(&props.node),
                    percent_encode_component(&props.storage)
                );
                let mut param = json!({});
                if let Some(content) = &props.content_filter {
                    param["content"] = content.to_string().into();
                }
                if let Some(vmid) = props.vmid_filter {
                    param["vmid"] = vmid.into();
                }
                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&url, Some(param)).await;
                    link.send_message(Msg::LoadResult(result));
                }));
            }
            Msg::LoadResult(result) => {
                self.data = Some(result.map_err(|err| err.to_string()));
            }
            Msg::ShowContentDialog(item) => {
                let controller = ctx.link().page_controller().unwrap();
                let volid = item.volid.clone();
                let content = VolumeActionDialog::new(item.clone())
                    .on_remove(ctx.link().callback({
                        let volid = volid.clone();
                        move |_| Msg::Remove(volid.clone())
                    }))
                    .on_show_config(
                        ctx.link()
                            .callback(move |_| Msg::ExtractConfig(volid.clone())),
                    );
                controller.show_modal_bottom_sheet(content);
            }
            Msg::Remove(volid) => {
                let link = ctx.link().clone();
                let url = format!(
                    "/nodes/{}/storage/{}/content/{}",
                    percent_encode_component(&props.node),
                    percent_encode_component(&props.storage),
                    percent_encode_component(&volid)
                );
                self.remove_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_delete_get(&url, None).await;
                    link.send_message(Msg::RemoveResult(result));
                }));
            }
            Msg::RemoveResult(result) => match result {
                Ok(_upid) => {
                    // TODO: check/track upid status?
                    ctx.link().send_message(Msg::Load);
                }
                Err(err) => crate::show_failed_command_error(ctx.link(), err),
            },
            Msg::ExtractConfig(volid) => {
                let param = json!({"volume": volid });
                let url = format!(
                    "/nodes/{}/vzdump/extractconfig",
                    percent_encode_component(&props.node)
                );
                let link = ctx.link().clone();
                self.extract_config_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(url, Some(param.clone())).await;
                    link.send_message(Msg::ExtractConfigResult(result));
                }));
            }
            Msg::ExtractConfigResult(result) => match result {
                Ok(config) => {
                    let content = Dialog::new(tr!("Configuration"))
                        .on_close(ctx.link().callback(|_| Msg::CloseConfigDialog))
                        .with_child(
                            Container::new()
                                .class(pwt::css::FlexFit)
                                .class("pwt-white-space-pre")
                                .padding(2)
                                .with_child(config),
                        );
                    self.config_dialog = Some(content.into());
                }
                Err(err) => crate::show_failed_command_error(ctx.link(), err),
            },
            Msg::CloseConfigDialog => {
                self.config_dialog = None;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        render_loaded_data(&self.data, |data| {
            if data.is_empty() {
                Container::new()
                    .padding(2)
                    .with_child(tr!("List is empty."))
                    .into()
            } else {
                self.view_content(ctx, data)
            }
        })
    }
}

impl From<StorageContentPanel> for VNode {
    fn from(props: StorageContentPanel) -> Self {
        let comp = VComp::new::<PveStorageContentPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
