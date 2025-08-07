use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;

use pwt::widget::{Column, Container, Dialog, Fa, List};
use pwt::AsyncAbortGuard;
use pwt::{prelude::*, widget::ListTile};

use proxmox_yew_comp::common_api_types::APTUpdateInfo;
use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use yew::virtual_dom::{Key, VComp, VNode};

use crate::widgets::title_subtitle_column;

#[derive(Clone, PartialEq, Properties)]
pub struct NodeUpdatesPanel {
    node: AttrValue,
}

impl NodeUpdatesPanel {
    pub fn new(node: impl Into<AttrValue>) -> Self {
        Self { node: node.into() }
    }
}

pub enum Msg {
    Load,
    LoadResult(Result<Vec<APTUpdateInfo>, Error>),
    ShowInfo(Option<APTUpdateInfo>),
}

pub struct PveNodeUpdatesPanel {
    data: Option<Result<Vec<APTUpdateInfo>, String>>,
    show_info: Option<APTUpdateInfo>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    //cmd_guard: Option<AsyncAbortGuard>,
}

impl PveNodeUpdatesPanel {
    fn view_updates(&self, ctx: &Context<Self>, data: &[APTUpdateInfo]) -> Html {
        let list: Vec<ListTile> = data
            .iter()
            .map(|s| {
                let from = s.old_version.clone().unwrap_or(tr!("not installed"));
                let version_info = format!("{from} -> {}", s.version);

                ListTile::new()
                    .key(Key::from(s.package.clone()))
                    .interactive(true)
                    .onclick({
                        let data = s.clone();
                        ctx.link()
                            .callback(move |_| Msg::ShowInfo(Some(data.clone())))
                    })
                    .with_child(title_subtitle_column(s.title.clone(), version_info))
                    .with_child(Fa::new("info-circle").large().padding_start(1))
            })
            .collect();

        List::new(list.len() as u64, move |pos| list[pos as usize].clone())
            .class(pwt::css::FlexFit)
            .grid_template_columns("1fr auto")
            .min_row_height(50)
            .into()
    }
}

impl Component for PveNodeUpdatesPanel {
    type Message = Msg;
    type Properties = NodeUpdatesPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
            show_info: None,
            reload_timeout: None,
            load_guard: None,
            // cmd_guard: None,
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Load => {
                let link = ctx.link().clone();
                let url = format!(
                    "/nodes/{}/apt/update",
                    percent_encode_component(&props.node)
                );
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
            Msg::ShowInfo(info) => {
                self.show_info = info;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        crate::widgets::render_loaded_data(&self.data, |data| {
            if data.is_empty() {
                Container::new()
                    .padding(2)
                    .with_child(tr!("List is empty."))
                    .into()
            } else {
                let info = self.show_info.as_ref().map(|info| {
                    Dialog::new(info.package.clone())
                        .with_child(
                            title_subtitle_column(info.title.clone(), info.version.clone())
                                .padding(2)
                                .with_child(
                                    Container::from_tag("p").with_child(info.description.clone()),
                                ),
                        )
                        .on_close(ctx.link().callback(|_| Msg::ShowInfo(None)))
                });

                Column::new()
                    .class(pwt::css::FlexFit)
                    .with_child(self.view_updates(ctx, data))
                    .with_optional_child(info)
                    .into()
            }
        })
    }
}

impl From<NodeUpdatesPanel> for VNode {
    fn from(props: NodeUpdatesPanel) -> Self {
        let comp = VComp::new::<PveNodeUpdatesPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
