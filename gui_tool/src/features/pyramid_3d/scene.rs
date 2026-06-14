mod pipeline;
mod primitive;

use iced::widget::shader;

use crate::features::pyramid_3d::scene::{pipeline::PyramidShape, primitive::PyramidPrimitive};

#[derive(Debug, Default)]
pub struct PyramidScene {
    pub pyramid_shape: PyramidShape,
}

impl<Message> shader::Program<Message> for PyramidScene {
    type State = ();

    type Primitive = PyramidPrimitive;

    fn draw(
        &self,
        state: &Self::State,
        cursor: iced::advanced::mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        todo!()
    }
}
