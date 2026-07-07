use std::f32::consts::PI;

use iced::Alignment::Center;
use iced::Element;
use iced::Length::Fill;
use iced::widget::{center, column, row, shader, slider, text};

use super::Msg;
use super::Pyramid;

pub fn view(pyramid: &Pyramid) -> Element<'_, Msg> {
    let controls = column![
        row![
            text("angle"),
            slider(0.0..=PI, pyramid.scene.angle, move |b| { Msg::RotationChanged(b) })
                .step(0.01)
                .width(100)
        ]
        .spacing(10)
    ]
    .spacing(10)
    .padding(20)
    .align_x(Center);

    let shader = shader(&pyramid.scene)
        .width(Fill)
        .height(Fill);
    center(column![shader, controls].align_x(Center)).into()
}
