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
            text("X轴"),
            slider(0.0..=2.0 * PI, pyramid.scene.angle_x, move |v| { Msg::RotationXChanged(v) })
                .step(0.01)
                .width(80)
        ]
        .spacing(10),
        row![
            text("Y轴"),
            slider(0.0..=2.0 * PI, pyramid.scene.angle_y, move |v| { Msg::RotationYChanged(v) })
                .step(0.01)
                .width(80)
        ]
        .spacing(10),
        row![
            text("Z轴"),
            slider(0.0..=2.0 * PI, pyramid.scene.angle_z, move |v| { Msg::RotationZChanged(v) })
                .step(0.01)
                .width(80)
        ]
        .spacing(10),
        row![
            text("缩放"),
            slider(0.1..=2.0, pyramid.scene.scale, move |v| { Msg::ScaleChanged(v) })
                .step(0.01)
                .width(80)
        ]
        .spacing(10),
    ]
    .spacing(8)
    .padding(20)
    .align_x(Center);

    let shader = shader(&pyramid.scene)
        .width(Fill)
        .height(Fill);
    center(column![shader, controls].align_x(Center)).into()
}
