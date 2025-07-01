mod top_nav_bar;
use pwt::props::PwtSpace;
pub use top_nav_bar::TopNavBar;

mod tasks_panel;
pub use tasks_panel::TasksPanel;

mod task_list_button;
pub use task_list_button::TasksListButton;

mod storage_content_panel;
pub use storage_content_panel::StorageContentPanel;

mod main_navigation;
pub use main_navigation::{MainNavigation, MainNavigationSelection};

mod volume_action_dialog;
pub use volume_action_dialog::show_volume_actions;

use pwt::prelude::*;
use pwt::widget::{Card, Column, Container, Fa, FieldLabel, ListTile, Progress, Row};

use proxmox_human_byte::HumanByte;
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

pub fn storage_card(
    storage: &str,
    storage_type: &str,
    shared: bool,
    storage_content: &str,
    total: Option<i64>,
    used: Option<i64>,
) -> Card {
    let (usage_text, percentage) = if let (Some(total), Some(used)) = (total, used) {
        let left_text = HumanByte::new_binary(used as f64);
        let right_text = HumanByte::new_binary(total as f64);

        (Row::new()
            .gap(2)
            .class("pwt-align-items-flex-end")
            .with_child(html! {
                <div class="pwt-font-size-title-small pwt-flex-fill">{left_text.to_string()}</div>
            })
            .with_flex_spacer()
            .with_child(html! {
                <div class="pwt-font-size-title-small">{right_text.to_string()}</div>
            }), (used as f32) / (total as f32))
    } else {
        (
            Row::new()
                .gap(2)
                .with_child(tr!("Storage size/usage unknown")),
            0.0,
        )
    };

    let usage = Column::new()
        .gap(1)
        .with_child(usage_text)
        .with_child(Progress::new().value(percentage));

    let content_text = Column::new()
        .gap(1)
        .class(pwt::css::Flex::Fill)
        .class(pwt::css::AlignItems::Center)
        .with_child(html! {<div class="pwt-font-size-title-medium">{format!("{storage} ({storage_type})")}</div>})
        .with_child(
            html! {<div class="pwt-font-size-title-small">{storage_content}</div>},
        );

    let type_icon = if shared { "cloud" } else { "folder" };

    let content = Row::new()
        .gap(2)
        .with_child(Fa::new(type_icon).large_2x().class("pwt-color-secondary"))
        .with_child(content_text);

    Card::new()
        .min_width(250)
        .class("pwt-interactive")
        .with_child(Column::new().gap(1).with_child(content).with_child(usage))
}

pub fn standard_list_tile(
    title: impl IntoPropValue<Option<AttrValue>>,
    subtitle: impl IntoPropValue<Option<AttrValue>>,
    leading: impl IntoPropValue<Option<Html>>,
    trailing: impl IntoPropValue<Option<Html>>,
) -> ListTile {
    let leading = leading.into_prop_value().unwrap_or(html! {<div/>});

    ListTile::new()
        .class(pwt::css::AlignItems::Center)
        .class("pwt-column-gap-2")
        .class("pwt-row-gap-1")
        //.class("pwt-scheme-surface")
        .border_bottom(true)
        .with_child(leading)
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

pub fn form_list_tile(
    title: impl Into<AttrValue>,
    subtitle: impl IntoPropValue<Option<AttrValue>>,
    trailing: impl IntoPropValue<Option<Html>>,
) -> ListTile {
    ListTile::new()
        .class(pwt::css::AlignItems::Center)
        .class("pwt-column-gap-2")
        .class("pwt-row-gap-1")
        //.class("pwt-scheme-surface")
        .border_bottom(true)
        .with_child({
            let mut column = Column::new().gap(1);

            if let Some(title) = title.into_prop_value() {
                column.add_child(
                    Container::new()
                        .class("pwt-font-size-title-medium")
                        .key("title")
                        .with_child(title.into()),
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

pub fn label_field(label: impl Into<AttrValue>, field: impl Into<Html>) -> Html {
    Column::new()
        .with_child(FieldLabel::new(label.into()).padding_bottom(PwtSpace::Em(0.3)))
        .with_child(field)
        .into()
}
