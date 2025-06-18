use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::widgets::TopNavBar;

use pwt::prelude::*;
use pwt::props::PwtSpace;

use pwt::widget::{Column, FieldLabel, LanguageSelector, ThemeDensitySelector, ThemeNameSelector};

#[derive(Clone, PartialEq, Properties)]
pub struct PageSettings {}

impl PageSettings {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct PvePageSettings {}

pub enum Msg {}

fn label_field(label: impl Into<AttrValue>, field: impl Into<VNode>) -> Html {
    Column::new()
        .with_child(FieldLabel::new(label.into()).padding_bottom(PwtSpace::Em(0.3)))
        .with_child(field)
        .into()
}

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
            .with_child(label_field(tr!("Language"), LanguageSelector::new()))
            .with_child(label_field(tr!("Theme"), ThemeNameSelector::new()))
            .with_child(label_field(tr!("Density"), ThemeDensitySelector::new()));

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
