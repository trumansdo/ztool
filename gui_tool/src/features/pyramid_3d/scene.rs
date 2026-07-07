mod camera;
mod pipeline;
mod primitive;

use pipeline::pyramid_shape::PyramidShape;
use primitive::PyramidPrimitive;

use iced::{Color, widget::shader::Program};

use crate::features::pyramid_3d::scene::camera::Camera;

#[derive(Debug, Clone)]
pub struct Scene {
    pub angle: f32,
    pub pyramid_shape: PyramidShape,
    pub camera: Camera,
    pub light_color: Color,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            angle: 0f32,
            pyramid_shape: PyramidShape::default(),
            camera: Camera::default(),
            light_color: Color::WHITE,
        }
    }
}

impl<Message> Program<Message> for Scene {
    type State = ();

    type Primitive = PyramidPrimitive;

    fn draw(
        &self,
        state: &Self::State,
        cursor: iced::advanced::mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        PyramidPrimitive::new(vec![self.pyramid_shape], &self.camera, bounds, self.light_color)
    }
}
