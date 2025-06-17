use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Column, LanguageSelector, ThemeDensitySelector, ThemeNameSelector};

use crate::widgets::TopNavBar;

#[derive(Clone, PartialEq, Properties)]
pub struct PageSettings {}

impl PageSettings {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct PvePageSettings {}

pub enum Msg {}

impl Component for PvePageSettings {
    type Message = Msg;
    type Properties = PageSettings;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = Column::new()
            .padding(2)
            .gap(2)
            .with_child(
                Column::new()
                    .gap(1)
                    .with_child(tr!("Language") + ":")
                    .with_child(LanguageSelector::new()),
            )
            .with_child(
                Column::new()
                    .gap(1)
                    .with_child(tr!("Theme") + ":")
                    .with_child(ThemeNameSelector::new()),
            )
            .with_child(
                Column::new()
                    .gap(1)
                    .with_child(tr!("Density") + ":")
                    .with_child(ThemeDensitySelector::new()),
            )
            .with_child("This is the settings page");

        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new().title("Settings").back("/"))
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageSettings {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageSettings>(Rc::new(self), None);
        VNode::from(comp)
    }
}
