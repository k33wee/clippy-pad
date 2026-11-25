use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, text};

use super::super::Applet;
use super::error_notice;
use crate::app::Message;
use crate::app::device::{ConnectedDevice, PairedRow};
use crate::app::style as controls;
use crate::fl;

pub(super) fn page(app: &Applet) -> Element<'_, Message> {
    let mut content = widget::Column::new().spacing(16).width(Length::Fill);
    if let Some(error) = &app.error {
        content = content.push(error_notice(error));
    }
    content = content
        .push(connected_section(app))
        .push(paired_section(app));
    content.into()
}

fn connected_section(app: &Applet) -> Element<'_, Message> {
    let mut section = widget::settings::section().title(fl!("connected-devices"));
    if app.devices.connected.is_empty() {
        section = section.add(text::caption(fl!("no-connected")));
    }
    for device in &app.devices.connected {
        section = section.add(
            widget::settings::item::builder(device.name.clone())
                .description(connected_description(device))
                .control(
                    controls::standard(fl!("disconnect").to_string()).on_press_maybe(
                        (!app.pending.contains(&device.addr))
                            .then(|| Message::Disconnect(device.addr.clone())),
                    ),
                ),
        );
    }
    section.into()
}

fn paired_section(app: &Applet) -> Element<'_, Message> {
    let mut section = widget::settings::section().title(fl!("paired-devices"));
    let rows = app.devices.paired_rows(&app.reconnecting);
    if rows.is_empty() {
        section = section.add(text::caption(fl!("no-paired")));
    }
    for row in rows {
        let renaming = app
            .rename
            .as_ref()
            .is_some_and(|rename| rename.addr == row.addr);
        let item: Element<'_, Message> = if renaming {
            rename_item(app)
        } else {
            widget::settings::item::builder(row.name.clone())
                .description(paired_description(&row))
                .control(paired_actions(app, &row))
                .into()
        };
        section = section.add(item);
    }
    if !app.reconnecting.is_empty() {
        section = section.add(text::caption(fl!("reconnect-hint")));
    }
    section.into()
}

fn connected_description(device: &ConnectedDevice) -> String {
    let battery = match device.battery {
        Some(charge) => format!("{charge}%"),
        None => fl!("battery-unknown").to_string(),
    };
    format!("{} • {}", device.addr, battery)
}

fn paired_description(row: &PairedRow) -> String {
    if row.connected {
        return format!("{} • {}", row.addr, fl!("connected"));
    }
    row.addr.clone()
}

fn paired_actions(app: &Applet, row: &PairedRow) -> Element<'static, Message> {
    if let Some(remaining) = row.reconnecting {
        return text::caption(fl!("reconnecting", seconds = remaining.as_secs())).into();
    }
    let idle = !app.pending.contains(&row.addr);
    let action = |name: &'static str, tooltip: String, message: Message| {
        widget::tooltip(
            widget::button::icon(widget::icon::from_name(name).handle())
                .extra_small()
                .on_press_maybe(idle.then_some(message)),
            text::body(tooltip),
            widget::tooltip::Position::Bottom,
        )
    };
    widget::Row::new()
        .spacing(4)
        .align_y(Alignment::Center)
        .push(action(
            "edit-symbolic",
            fl!("rename").to_string(),
            Message::RenameStart(row.addr.clone()),
        ))
        .push(action(
            "view-refresh-symbolic",
            fl!("reconnect").to_string(),
            Message::Reconnect(row.addr.clone()),
        ))
        .push(action(
            "user-trash-symbolic",
            fl!("remove").to_string(),
            Message::Remove(row.addr.clone()),
        ))
        .into()
}

fn rename_item(app: &Applet) -> Element<'_, Message> {
    let value = app
        .rename
        .as_ref()
        .map(|rename| rename.value.as_str())
        .unwrap_or_default();
    let input = widget::text_input(fl!("rename-placeholder"), value)
        .on_input(Message::RenameInput)
        .on_submit(|_| Message::RenameSubmit)
        .width(Length::Fixed(160.0));
    widget::settings::item::builder(fl!("rename"))
        .control(
            widget::Row::new()
                .spacing(8)
                .push(input)
                .push(controls::suggested(fl!("save").to_string()).on_press(Message::RenameSubmit))
                .push(
                    controls::standard(fl!("cancel").to_string()).on_press(Message::RenameCancel),
                ),
        )
        .into()
}
