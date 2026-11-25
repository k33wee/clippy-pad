mod about;
mod devices;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, container, text};
use cosmic::{Element, theme};

use super::{Applet, Message, Page};
use crate::app::style as controls;
use crate::fl;

pub(super) fn popup(app: &Applet) -> Element<'_, Message> {
    let mut content = widget::Column::new()
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);
    if app.page == Page::Devices {
        content = content.push(header());
    }
    let body = match app.page {
        Page::Devices => devices::page(app),
        Page::About => about::page(),
    };
    content = content
        .push(
            widget::scrollable(body)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .push(widget::divider::horizontal::light())
        .push(footer(app));
    container(content)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn header() -> Element<'static, Message> {
    widget::Row::new()
        .height(32)
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .push(widget::Space::new().width(Length::Fill))
        .push(widget::tooltip(
            icon_button("help-about-symbolic", Message::ShowAbout),
            text::body(fl!("about")),
            widget::tooltip::Position::Bottom,
        ))
        .into()
}

fn icon_button(
    name: &'static str,
    message: Message,
) -> widget::button::IconButton<'static, Message> {
    widget::button::icon(widget::icon::from_name(name).handle())
        .width(32)
        .height(32)
        .padding(6)
        .on_press(message)
}

fn footer(app: &Applet) -> Element<'_, Message> {
    if app.page != Page::Devices {
        return back_button();
    }
    let mut row = widget::Row::new()
        .spacing(12)
        .width(Length::Fill)
        .align_y(Alignment::Center);
    if app.loading {
        row = row.push(text::caption(fl!("loading")));
    } else {
        row = row.push(
            controls::standard(fl!("refresh").to_string())
                .on_press_maybe(app.pending.is_empty().then_some(Message::Refresh)),
        );
    }
    row.push(text::caption(fl!("refresh-hint"))).into()
}

fn back_button() -> Element<'static, Message> {
    widget::button::text(fl!("back"))
        .leading_icon(widget::icon::from_name("go-previous-symbolic").handle())
        .on_press(Message::ShowHome)
        .into()
}

pub(super) fn error_notice(error: &str) -> Element<'static, Message> {
    container(
        widget::Column::new()
            .spacing(8)
            .push(text::body(error.to_string()))
            .push(controls::text(fl!("dismiss").to_string()).on_press(Message::DismissError)),
    )
    .padding(12)
    .width(Length::Fill)
    .class(theme::Container::Card)
    .into()
}
