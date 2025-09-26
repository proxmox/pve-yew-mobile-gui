use pwt::prelude::*;
use pwt::widget::form::Number;

use pwt::props::{FieldBuilder, WidgetBuilder};
use pwt_macros::widget;

#[widget(comp=PveVlanFieldComp, @input)]
#[derive(Clone, Properties, PartialEq)]
pub struct PveVlanField {}

impl PveVlanField {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn get_std_label() -> String {
        tr!("VLAN Tag")
    }
}
pub struct PveVlanFieldComp {}

impl Component for PveVlanFieldComp {
    type Message = ();
    type Properties = PveVlanField;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        Number::<u16>::new()
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .placeholder(tr!("no VLAN"))
            .min(1)
            .max(4094)
            .into()
    }
}
