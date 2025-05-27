use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::VList;

use pwt::prelude::*;
use pwt::widget::{Column, Container};

use pwt_macros::{builder, widget};

#[widget(comp=PwtListTile, @element)]
#[builder]
#[derive(Clone, PartialEq, Properties)]
pub struct ListTile {
    /// The primary content of the list tile.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    title: Option<AttrValue>,
    /// Additional content diplayed below the tile.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    subtitle: Option<AttrValue>,

    /// Leading content
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    leading: Option<Html>,

    /// Trailing content
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    trailing: Option<Html>,

    /// Disable flag
    #[prop_or_default]
    #[builder]
    pub disabled: bool,

    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    /// Activate callback (click, enter, space)
    pub on_tab: Option<Callback<()>>,
}

impl ListTile {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

#[doc(hidden)]
pub struct PwtListTile {}

impl Component for PwtListTile {
    type Message = ();
    type Properties = ListTile;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut text = VList::new();
        if let Some(title) = &props.title {
            text.push(html! { <p class="pwt-font-body-large">{title}</p>});
        }
        if let Some(subtitle) = &props.subtitle {
            text.push(html! { <p class="pwt-font-body-medium">{subtitle}</p>});
        }

        let middle = Column::new().class("pwt-flex-fill").with_child(text);

        let interactive = props.on_tab.is_some();
        let disabled = props.disabled;

        let mut tile = Container::new()
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .attribute("disabled", props.disabled.then(|| ""))
            .class("pwt-list-tile")
            .class(interactive.then(|| "pwt-interactive"))
            .onclick({
                let on_tab = props.on_tab.clone();
                move |_| {
                    if let Some(on_tab) = &on_tab {
                        if interactive && !disabled {
                            on_tab.emit(());
                        }
                    }
                }
            });

        if let Some(leading) = &props.leading {
            tile.add_child(
                Container::new()
                    .class("pwt-list-tile-leading")
                    .with_child(leading.clone()),
            );
        }

        tile.add_child(middle);

        if let Some(trailing) = &props.trailing {
            tile.add_child(
                Container::new()
                    .class("pwt-list-tile-trailing")
                    .with_child(trailing.clone()),
            );
        }

        tile.into()
    }
}
