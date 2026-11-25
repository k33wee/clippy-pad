//! Keep native COSMIC colors and focus states without pill-shaped controls.

use cosmic::iced::Length;
use cosmic::theme::Button;
use cosmic::widget;
use cosmic::widget::button::{Catalog, Style};

#[derive(Clone, Copy)]
enum Kind {
    Standard,
    Suggested,
    Text,
}

impl Kind {
    fn class(self) -> Button {
        match self {
            Self::Standard => Button::Standard,
            Self::Suggested => Button::Suggested,
            Self::Text => Button::Text,
        }
    }
}

fn square(mut style: Style) -> Style {
    style.border_radius = 4.0.into();
    style
}

fn control<Message: Clone + 'static>(
    label: String,
    kind: Kind,
) -> widget::button::Button<'static, Message> {
    widget::button::custom(widget::text::body(label.clone()))
        .name(label)
        .padding([6, 12])
        .height(Length::Fixed(36.0))
        .class(Button::Custom {
            active: Box::new(move |focused, theme| {
                // `selected` stays false. These buttons are actions, not toggles.
                square(theme.active(focused, false, &kind.class()))
            }),
            hovered: Box::new(move |focused, theme| {
                square(theme.hovered(focused, false, &kind.class()))
            }),
            pressed: Box::new(move |focused, theme| {
                square(theme.pressed(focused, false, &kind.class()))
            }),
            disabled: Box::new(move |theme| square(theme.disabled(&kind.class()))),
        })
}

pub fn standard<Message: Clone + 'static>(
    label: String,
) -> widget::button::Button<'static, Message> {
    control(label, Kind::Standard)
}

pub fn suggested<Message: Clone + 'static>(
    label: String,
) -> widget::button::Button<'static, Message> {
    control(label, Kind::Suggested)
}

pub fn text<Message: Clone + 'static>(label: String) -> widget::button::Button<'static, Message> {
    control(label, Kind::Text)
}
