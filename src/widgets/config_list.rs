use std::rc::Rc;

use anyhow::Error;
use pwt::props::{IntoSubmitCallback, RenderFn, SubmitCallback};
use pwt::touch::SideDialog;
use pwt::widget::form::{Form, FormContext, ResetButton, SubmitButton};
use pwt::widget::{Column, Container, List, Row};
use pwt::{impl_class_prop_builder, impl_yew_std_props_builder};
use pwt::{prelude::*, AsyncPool};

use yew::html::{IntoPropValue, Scope};
use yew::virtual_dom::{Key, VComp, VNode};

use pwt_macros::builder;

use crate::show_failed_command_error;
use crate::widgets::form_list_tile;

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct ConfigTile {
    /// Use single line layout (title/value in one row).
    #[prop_or_default]
    #[builder]
    pub single_row: bool,

    /// Titel text
    pub title: AttrValue,

    /// Value as text
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub value_text: Option<AttrValue>,

    /// Placeholder (displayed when there is no value)
    #[prop_or(AttrValue::Static("-"))]
    pub placeholder: AttrValue,

    /// Edit input panel builder
    #[prop_or_default]
    pub create_input_panel: Option<RenderFn<FormContext>>,

    /// Submit callback.
    #[prop_or_default]
    pub on_submit: Option<SubmitCallback<FormContext>>,
}

impl ConfigTile {
    pub fn new(title: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            title: title.into(),
        })
    }
    pub fn new_bool(title: impl Into<AttrValue>, value: impl IntoPropValue<Option<bool>>) -> Self {
        let value: Option<bool> = value.into_prop_value();
        yew::props!(Self {
            title: title.into(),
        })
        .single_row(true)
        .value_text(value.map(|v| if v { tr!("Yes") } else { tr!("No") }))
    }

    pub fn create_input_panel(mut self, renderer: impl 'static + Fn(&FormContext) -> Html) -> Self {
        self.create_input_panel = Some(RenderFn::new(renderer));
        self
    }

    pub fn on_submit(mut self, callback: impl IntoSubmitCallback<FormContext>) -> Self {
        self.on_submit = callback.into_submit_callback();
        self
    }
}

#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct ConfigList {
    /// Yew component `ref`.
    #[prop_or_default]
    pub node_ref: NodeRef,

    /// Yew `key` property
    #[prop_or_default]
    pub key: Option<Key>,

    /// CSS class
    #[prop_or_default]
    pub class: Classes,

    /// Add a line as separator between list items.
    #[prop_or_default]
    #[builder]
    pub separator: bool,

    tiles: Vec<ConfigTile>, // fixme: use Rc ?
}

impl ConfigList {
    pub fn new() -> Self {
        yew::props!(Self { tiles: Vec::new() })
    }

    impl_yew_std_props_builder!();
    impl_class_prop_builder!();

    pub fn with_tile(mut self, tile: impl Into<ConfigTile>) -> Self {
        self.add_tile(tile);
        self
    }

    pub fn add_tile(&mut self, tile: impl Into<ConfigTile>) {
        self.tiles.push(tile.into());
    }

    pub fn tiles(mut self, tiles: impl IntoIterator<Item = ConfigTile>) -> Self {
        self.add_tiles(tiles);
        self
    }

    pub fn add_tiles(&mut self, tiles: impl IntoIterator<Item = ConfigTile>) {
        self.tiles.extend(tiles);
    }
}

pub struct ProxmoxConfigList {
    edit_dialog: Option<Html>,
    loading: bool,
    async_pool: AsyncPool,
}

pub enum Msg {
    CloseDialog,
    ShowDialog(Html),
    Submit(FormContext, Option<SubmitCallback<FormContext>>),
    SubmitResult(Result<(), Error>),
}

impl ProxmoxConfigList {
    fn create_edit_dialog(link: &Scope<Self>, tile: ConfigTile) -> SideDialog {
        let form_ctx = FormContext::new();

        let input_panel = Column::new()
            .gap(1)
            .class(pwt::css::Flex::Fill)
            .class(pwt::css::AlignItems::Stretch)
            .class("pwt-font-size-title-medium")
            .with_child(tile.title.clone())
            .with_flex_spacer()
            .with_child(tile.create_input_panel.unwrap().apply(&form_ctx));

        let on_submit = tile.on_submit;

        let form = Form::new()
            .form_context(form_ctx.clone())
            .class(pwt::css::FlexFit)
            .with_child(
                Column::new()
                    .class(pwt::css::FlexFit)
                    .padding(2)
                    .gap(4)
                    .with_child(input_panel)
                    .with_child(
                        Row::new()
                            .gap(2)
                            .with_flex_spacer()
                            .with_child(ResetButton::new().class("pwt-button-text"))
                            .with_child(SubmitButton::new().text(tr!("Update")).on_submit(
                                link.callback(move |_| {
                                    Msg::Submit(form_ctx.clone(), on_submit.clone())
                                }),
                            )),
                    ),
            );

        SideDialog::new()
            .location(pwt::touch::SideDialogLocation::Bottom)
            .on_close(link.callback(|_| Msg::CloseDialog))
            .with_child(form)
    }
}

impl Component for ProxmoxConfigList {
    type Message = Msg;
    type Properties = ConfigList;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            edit_dialog: None,
            loading: false,
            async_pool: AsyncPool::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ShowDialog(dialog) => {
                self.edit_dialog = Some(dialog);
            }
            Msg::CloseDialog => {
                self.edit_dialog = None;
            }
            Msg::Submit(form_ctx, on_submit) => {
                if let Some(on_submit) = on_submit.clone() {
                    let link = ctx.link().clone();
                    self.loading = true;
                    self.async_pool.spawn(async move {
                        let result = on_submit.apply(form_ctx).await;
                        link.send_message(Msg::SubmitResult(result));
                    });
                }
            }
            Msg::SubmitResult(result) => {
                self.loading = false;
                match result {
                    Ok(()) => self.edit_dialog = None,
                    Err(err) => show_failed_command_error(ctx.link(), err),
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        Container::new()
            .class(props.class.clone())
            .node_ref(props.node_ref.clone())
            .with_child({
                let list = props.tiles.clone();
                let link = ctx.link().clone();

                List::new(list.len() as u64, move |pos| {
                    let tile = list[pos as usize].clone();

                    let value_text = tile
                        .value_text
                        .clone()
                        .unwrap_or_else(|| tile.placeholder.clone());

                    let mut list_tile = if tile.single_row {
                        let trailing = Container::new()
                            .style("text-align", "end")
                            .with_child(value_text);
                        form_list_tile(tile.title.clone(), None::<&str>, Some(trailing.into()))
                    } else {
                        form_list_tile(tile.title.clone(), value_text, None)
                    };

                    if tile.create_input_panel.is_some() {
                        list_tile.set_interactive(true);
                        list_tile.add_onclick({
                            let link = link.clone();
                            let tile = tile.clone();
                            move |_| {
                                if tile.create_input_panel.is_some() {
                                    link.send_message(Msg::ShowDialog(
                                        Self::create_edit_dialog(&link, tile.clone()).into(),
                                    ));
                                }
                            }
                        });
                    }
                    list_tile
                })
                .virtual_scroll(Some(false))
                .separator(props.separator)
                .class("pwt-fit")
                .grid_template_columns("1fr auto")
            })
            .with_optional_child(self.edit_dialog.clone())
            .into()
    }
}

impl From<ConfigList> for VNode {
    fn from(props: ConfigList) -> Self {
        let key = props.key.clone();
        let comp = VComp::new::<ProxmoxConfigList>(Rc::new(props), key);
        VNode::from(comp)
    }
}
