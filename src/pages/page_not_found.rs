use pwt::prelude::*;
use pwt::widget::{error_message, Column};

use crate::widgets::TopNavBar;

#[function_component]
pub fn PageNotFound() -> Html {
    let content = error_message("page not found").padding(2);

    Column::new()
        .class("pwt-fit")
        .with_child(TopNavBar::new())
        .with_child(content)
        .into()
}
