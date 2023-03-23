use pwt::prelude::*;
use pwt::widget::{error_message, Column};

use crate::TopNavBar;

#[function_component]
pub fn PageNotFound() -> Html {
    let content = error_message("page not found", "pwt-p-2");

    Column::new()
        .class("pwt-viewport")
        .with_child(TopNavBar::new())
        .with_child(content)
        .into()
}
