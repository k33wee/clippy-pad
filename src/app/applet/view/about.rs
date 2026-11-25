use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, text};

use crate::app::Message;
use crate::fl;

pub(super) fn page() -> Element<'static, Message> {
    let identity = widget::Column::new()
        .spacing(8)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .push(
            widget::icon::from_svg_bytes(include_bytes!("../../../../resources/icon.svg"))
                .symbolic(true)
                .icon()
                .size(48),
        )
        .push(text::title3(fl!("app-title")))
        .push(text::caption(format!(
            "{} {}",
            fl!("version"),
            env!("CARGO_PKG_VERSION")
        )))
        .push(text::body(fl!("about-text")).align_x(Alignment::Center));

    widget::Column::new()
        .spacing(12)
        .width(Length::Fill)
        .push(text::title4(fl!("about")))
        .push(
            widget::container(identity)
                .width(Length::Fill)
                .align_x(Alignment::Center),
        )
        .push(widget::divider::horizontal::light())
        .push(
            widget::button::link("GPL-3.0-or-later").on_press(Message::OpenLink(
                "https://www.gnu.org/licenses/gpl-3.0.html",
            )),
        )
        .into()
}
