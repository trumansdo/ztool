use iced::Pixels;

pub const FONT_SIZE: f32 = 14.0;
pub const COMPONENT_SIZE: f32 = 14.0;

pub fn font(multiplier: f32) -> f32 {
    FONT_SIZE * multiplier
}

pub fn size(multiplier: f32) -> Pixels {
    Pixels(COMPONENT_SIZE * multiplier)
}

pub fn padding(multiplier: f32) -> iced::Padding {
    let p = size(multiplier).0;
    iced::Padding::from([p, p])
}

pub fn padding2(v: f32, h: f32) -> iced::Padding {
    iced::Padding::from([size(v).0, size(h).0])
}
