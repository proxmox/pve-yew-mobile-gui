mod top_nav_bar;
pub use top_nav_bar::TopNavBar;

mod vm_config_panel;
pub use vm_config_panel::VmConfigPanel;

mod tasks_panel;
pub use tasks_panel::TasksPanel;

use pwt::prelude::*;
use pwt::widget::{Column, Container, Fa, ListTile};

use yew::html::IntoPropValue;

pub fn standard_list_tile(
    title: impl IntoPropValue<Option<AttrValue>>,
    subtitle: impl IntoPropValue<Option<AttrValue>>,
    leading: impl IntoPropValue<Option<Html>>,
    trailing: impl IntoPropValue<Option<Html>>,
) -> ListTile {
    ListTile::new()
        .class(pwt::css::AlignItems::Center)
        .class("pwt-gap-2")
        .class("pwt-scheme-surface")
        .border_top(true)
        .with_optional_child(leading.into_prop_value())
        .with_child({
            let mut column = Column::new().gap(1);

            if let Some(title) = title.into_prop_value() {
                column.add_child(
                    Container::new()
                        .class("pwt-font-size-title-medium")
                        .key("title")
                        .with_child(title),
                );
            }

            if let Some(subtitle) = subtitle.into_prop_value() {
                column.add_child(
                    Container::new()
                        .class("pwt-font-size-title-small")
                        .key("subtitle")
                        .with_child(subtitle),
                );
            }
            column
        })
        .with_optional_child(trailing.into_prop_value())
}

pub fn icon_list_tile(
    icon: impl Into<Fa>,
    title: impl IntoPropValue<Option<AttrValue>>,
    subtitle: impl IntoPropValue<Option<AttrValue>>,
    trailing: impl IntoPropValue<Option<Html>>,
) -> ListTile {
    let icon = icon.into().fixed_width().large_2x();
    standard_list_tile(title, subtitle, icon.into_html(), trailing)
}
