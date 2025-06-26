use std::rc::Rc;

use anyhow::Error;
use serde_json::Value;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::widget::{Column, Container, Progress};
use pwt::{prelude::*, AsyncAbortGuard};

use crate::widgets::TopNavBar;
use crate::StorageEntry;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

#[derive(Clone, PartialEq, Properties)]
pub struct PageStorageStatus {
    node: AttrValue,
    name: AttrValue,
}

impl PageStorageStatus {
    pub fn new(node: impl Into<AttrValue>, name: impl Into<AttrValue>) -> Self {
        Self {
            node: node.into(),
            name: name.into(),
        }
    }
}

pub struct PvePageStorageStatus {
    status: Option<Result<Value, String>>,
    load_guard: Option<AsyncAbortGuard>,
}

pub enum Msg {
    Load,
    LoadResult(Result<Value, Error>),
}

impl Component for PvePageStorageStatus {
    type Message = Msg;
    type Properties = PageStorageStatus;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            status: None,
            load_guard: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            Msg::Load => {
                let link = ctx.link().clone();

                let url = format!(
                    "/nodes/{}/storage/{}/status",
                    percent_encode_component(&props.node),
                    percent_encode_component(&props.name)
                );

                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&url, None).await;
                    link.send_message(Msg::LoadResult(result));
                }));
            }
            Msg::LoadResult(result) => {
                self.status = Some(result.map_err(|err| err.to_string()));
            }
        }
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content: Html = match &self.status {
            Some(Ok(status)) => Container::new()
                .padding(2)
                .with_child(format!("This is the status for storage {}", props.name))
                .into(),
            Some(Err(err)) => pwt::widget::error_message(err).into(),
            None => Progress::new().class("pwt-delay-visibility").into(),
        };

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title(format!("Storage {}", props.name))
                    .back(true),
            )
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageStorageStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageStorageStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}
