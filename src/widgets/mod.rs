mod top_nav_bar;
pub use top_nav_bar::TopNavBar;

mod vm_config_panel;
pub use vm_config_panel::VmConfigPanel;

mod tasks_panel;
pub use tasks_panel::TasksPanel;

use pwt::prelude::*;
use pwt::widget::{Card, Column, Container, Fa, ListTile, Progress, Row};

use yew::html::IntoPropValue;

pub fn standard_card(
    title: impl Into<AttrValue>,
    subtitle: impl IntoPropValue<Option<AttrValue>>,
) -> Card {
    let title = title.into();

    let head: Html = match subtitle.into_prop_value() {
        Some(subtitle) => Column::new()
            .padding(2)
            .gap(1)
            .border_bottom(true)
            .with_child(html! {
                <div class="pwt-font-size-title-large">{title}</div>
            })
            .with_child(html! {
                <div class="pwt-font-size-title-small">{subtitle}</div>
            })
            .into(),
        None => Container::new()
            .border_bottom(true)
            .padding(2)
            .class("pwt-font-size-title-large")
            .with_child(title)
            .into(),
    };

    Card::new()
        .padding(0)
        .class("pwt-flex-none pwt-overflow-hidden")
        .with_child(head)
}

pub fn standard_list_tile(
    title: impl IntoPropValue<Option<AttrValue>>,
    subtitle: impl IntoPropValue<Option<AttrValue>>,
    leading: impl IntoPropValue<Option<Html>>,
    trailing: impl IntoPropValue<Option<Html>>,
) -> ListTile {
    ListTile::new()
        .class(pwt::css::AlignItems::Center)
        .class("pwt-gap-1")
        //.class("pwt-scheme-surface")
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

pub fn list_tile_usage(
    left_text: impl Into<AttrValue>,
    right_text: impl Into<AttrValue>,
    percentage: f32,
) -> Column {
    let progress = Progress::new().value(percentage);

    let text_row = Row::new()
        .gap(2)
        .class("pwt-align-items-flex-end")
        .with_child(html! {
            <div class="pwt-font-size-title-small pwt-flex-fill">{left_text.into()}</div>
        })
        .with_child(html! {
            <div class="pwt-font-size-title-small">{right_text.into()}</div>
        });

    Column::new()
        .gap(1)
        .style("grid-column", "1 / -1")
        .with_child(text_row)
        .with_child(progress)
}
