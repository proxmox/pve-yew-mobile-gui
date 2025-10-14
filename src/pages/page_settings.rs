use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::widgets::TopNavBar;

use pwt::prelude::*;

use pwt::widget::{Column, LanguageSelector, ThemeDensitySelector, ThemeNameSelector};

use proxmox_yew_comp::layout::mobile_form::label_widget;

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

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let content = Column::new()
            .padding(2)
            .gap(2)
            .with_child(label_widget(tr!("Language"), LanguageSelector::new()))
            .with_child(label_widget(tr!("Theme"), ThemeNameSelector::new()))
            .with_child(label_widget(tr!("Density"), ThemeDensitySelector::new()));

        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new().title("Settings").back(true))
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
