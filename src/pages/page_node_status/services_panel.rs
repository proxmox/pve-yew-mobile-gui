use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;

use pwt::widget::{Column, Container, List, Progress};
use pwt::AsyncAbortGuard;
use pwt::{prelude::*, widget::ListTile};

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};
use yew::virtual_dom::{VComp, VNode};

use crate::api_types::ServiceStatus;
use crate::widgets::title_subtitle_column;

#[derive(Clone, PartialEq, Properties)]
pub struct NodeServicesPanel {
    node: AttrValue,
    standalone: bool,
}

impl NodeServicesPanel {
    pub fn new(node: impl Into<AttrValue>, standalone: bool) -> Self {
        Self {
            node: node.into(),
            standalone,
        }
    }
}

pub enum Msg {
    Load,
    LoadResult(Result<Vec<ServiceStatus>, Error>),
}

pub struct PveNodeServicesPanel {
    data: Option<Result<Vec<ServiceStatus>, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    //cmd_guard: Option<AsyncAbortGuard>,
}

fn service_state_icon(s: &ServiceStatus) -> Container {
    if s.state == "running" {
        Container::new()
            .class("pwt-color-primary")
            .style("text-align", "end")
            .with_child(&s.state)
    } else if (s.unit_state == "masked") || (s.unit_state == "not-found") {
        Container::new()
            .style("opacity", "50%")
            .style("text-align", "end")
            .with_child(&s.unit_state)
    } else {
        Container::new()
            .with_child(&s.state)
            .style("text-align", "end")
    }
}

impl PveNodeServicesPanel {
    fn view_service_summary(&self, ctx: &Context<Self>, data: &[ServiceStatus]) -> Html {
        let props = ctx.props();
        let standalone = props.standalone;

        let all_running = !data.iter().any(|s| {
            if (s.name == "corosync") && standalone {
                return false;
            }
            // TODO: only one NTP provider will be active
            s.state != "running" && !(s.unit_state == "masked" || s.unit_state == "not-found")
        });

        let msg = if all_running {
            tr!("All required services running")
        } else {
            tr!("One or more required services not running")
        };

        title_subtitle_column(msg, None::<&str>).padding(2).into()
    }

    fn view_services(&self, _ctx: &Context<Self>, data: &[ServiceStatus]) -> Html {
        let list: Vec<ListTile> = data
            .iter()
            .map(|s| {
                ListTile::new()
                    .with_child(title_subtitle_column(s.name.clone(), s.desc.clone()))
                    .with_child(service_state_icon(&s))
            })
            .collect();

        List::new(list.len() as u64, move |pos| list[pos as usize].clone())
            .class(pwt::css::FlexFit)
            .grid_template_columns("1fr auto")
            .into()
    }
}

impl Component for PveNodeServicesPanel {
    type Message = Msg;
    type Properties = NodeServicesPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
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
                let url = format!("/nodes/{}/services", percent_encode_component(&props.node));
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
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        crate::widgets::render_loaded_data(&self.data, |data| {
            Column::new()
                .class(pwt::css::FlexFit)
                .with_child(self.view_service_summary(ctx, data))
                .with_child(self.view_services(ctx, data))
                .into()
        })
    }
}

impl From<NodeServicesPanel> for VNode {
    fn from(props: NodeServicesPanel) -> Self {
        let comp = VComp::new::<PveNodeServicesPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}
