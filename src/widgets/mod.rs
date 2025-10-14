mod top_nav_bar;
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
pub use volume_action_dialog::VolumeActionDialog;

mod guest_backup_panel;
pub use guest_backup_panel::GuestBackupPanel;

use pwt::prelude::*;
use pwt::props::IntoOptionalInlineHtml;
use pwt::widget::{Card, Column, Container, Fa, Progress, Row};

use proxmox_human_byte::HumanByte;

pub fn standard_card(
    title: impl Into<AttrValue>,
    subtitle: impl IntoOptionalInlineHtml,
    trailing: impl IntoOptionalInlineHtml,
) -> Card {
    let title = title.into();

    let head: Html = match subtitle.into_optional_inline_html() {
        Some(subtitle) => Column::new()
            .gap(1)
            .with_child(html! {
                <div class="pwt-font-size-title-large">{title}</div>
            })
            .with_child(html! {
                <div class="pwt-font-size-title-small">{subtitle}</div>
            })
            .into(),
        None => Container::new()
            .class("pwt-font-size-title-large")
            .with_child(title)
            .into(),
    };

    let mut row = Row::new()
        .class(pwt::css::AlignItems::Center)
        .padding(2)
        .border_bottom(true)
        .gap(1)
        .with_child(head);
    if let Some(trailing) = trailing.into_optional_inline_html() {
        row.add_flex_spacer();
        row.add_child(trailing);
    }
    Card::new()
        .padding(0)
        .class("pwt-flex-none pwt-overflow-hidden")
        .with_child(row)
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
