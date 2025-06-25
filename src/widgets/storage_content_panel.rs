use std::rc::Rc;

use anyhow::Error;
use proxmox_human_byte::HumanByte;
use serde::{Deserialize, Serialize};
use serde_json::json;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Container, Fa, List, ListTile, Progress};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::StorageContent;

use crate::widgets::icon_list_tile;

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

// fixme: define in pve-api-types
#[derive(Deserialize, Serialize)]
pub struct StorageEntry {
    format: String,
    size: i64,
    volid: String,
}

pub enum Msg {
    Load,
    LoadResult(Result<Vec<StorageEntry>, Error>),
}

pub struct PveStorageContentPanel {
    data: Option<Result<Vec<StorageEntry>, String>>,
    load_guard: Option<AsyncAbortGuard>,
}

impl PveStorageContentPanel {
    fn view_list(&self, _ctx: &Context<Self>, data: &[StorageEntry]) -> Html {
        let mut list: Vec<ListTile> = Vec::new();

        for item in data {
            list.push(icon_list_tile(
                Fa::new("cloud"),
                item.volid.clone(),
                format!("Size {}", HumanByte::new_binary(item.size as f64)),
                None,
            ));
        }

        List::new(list.len() as u64, move |pos| list[pos as usize].clone())
            .class(pwt::css::FlexFit)
            .grid_template_columns("auto 1fr auto")
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
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.data {
            Some(Ok(data)) if !data.is_empty() => self.view_list(ctx, data),
            Some(Ok(_data)) => Container::new()
                .padding(2)
                .with_child(tr!("List is empty."))
                .into(),
            Some(Err(err)) => pwt::widget::error_message(err).into(),
            None => Progress::new().class("pwt-delay-visibility").into(),
        }
    }
}

impl From<StorageContentPanel> for VNode {
    fn from(props: StorageContentPanel) -> Self {
        let comp = VComp::new::<PveStorageContentPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
