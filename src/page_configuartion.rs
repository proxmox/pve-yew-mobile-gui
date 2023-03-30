use std::rc::Rc;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::Column;
use pwt::widget::nav_menu::{MenuItem, NavigationMenu};

use crate::{TopNavBar, ListTile};

#[derive(Clone, PartialEq, Properties)]
pub struct PageConfiguration {}

impl PageConfiguration {
    pub fn new() -> Self {
        Self {}
    }
}
pub struct PvePageConfiguration {
}

impl PvePageConfiguration {

    fn create_menu(&self, ctx: &Context<Self>) -> Html {
        Column::new()
            .with_child(
                ListTile::new()
                    .leading(html!{<i class="fa fa-trash"/>})
                    .title("TEST")
                    .subtitle("This is a stupid test")
            )
            .into()
    }

}

impl Component for PvePageConfiguration {
    type Message = ();
    type Properties = PageConfiguration;

    fn create(ctx: &Context<Self>) -> Self {

        Self {}
    }

      fn view(&self, ctx: &Context<Self>) -> Html {

        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new().title("Configuration"))
            .with_child(self.create_menu(ctx))
            .into()
    }
}

impl Into<VNode> for PageConfiguration {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageConfiguration>(Rc::new(self), None);
        VNode::from(comp)
    }
}
