use std::rc::Rc;

use yew::virtual_dom::{VComp, VList, VNode};
use yew::html::IntoPropValue;

use pwt::prelude::*;
use pwt::widget::{Column, Container, Row};

use pwt_macros::{widget, builder};

#[widget(comp=PwtListTile, @element)]
#[builder]
#[derive(Clone, PartialEq, Properties)]
pub struct ListTile {
    /// The primary content of the list tile.
    #[builder(IntoPropValue, into_prop_value)]
    title: Option<AttrValue>,
    /// Additional content diplayed below the tile.
    #[builder(IntoPropValue, into_prop_value)]
    subtitle: Option<AttrValue>,

    /// Leading content
    #[builder(IntoPropValue, into_prop_value)]
    leading: Option<Html>,

    /// Trailing content
    #[builder(IntoPropValue, into_prop_value)]
    trailing: Option<Html>,
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

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut text = VList::new();
        if let Some(title) = &props.title {
            text.push(html!{ <p class="pwt-font-body-large">{title}</p>});
        }
        if let Some(subtitle) = &props.subtitle {
            text.push(html!{ <p class="pwt-font-body-medium">{subtitle}</p>});
        }

        let middle = Column::new()
            .class("pwt-flex-fill")
            .with_child(text);

        let mut tile = Container::new()
            .with_std_props(&props.std_props)
            .class("pwt-d-flex") // fixme: remove
            .class("pwt-gap-2") // fixme: remove
            .class("pwt-align-items-center") // fixme: remove
            .class("pwt-list-tile")
            .padding_x(2)
            .padding_y(1);

        if let Some(leading) = &props.leading {
            tile.add_child(
                Container::new()
                    .class("pwt-list-tile-leading")
                    .with_child(leading.clone())
            );
        }

        tile.add_child(middle);

        if let Some(trailing) = &props.trailing {
            tile.add_child(
                Container::new()
                    .class("pwt-list-tile-trailing")
                    .with_child(trailing.clone())
            );
        }

        tile.into()
    }
}
