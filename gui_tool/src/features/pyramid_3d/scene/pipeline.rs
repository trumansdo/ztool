mod pyramid_shape;
mod vertex;

pub use pyramid_shape::PyramidShape;
pub use vertex::Vertex;

use iced::widget::shader;

pub struct PyramidPipeline {}

impl shader::Pipeline for PyramidPipeline {
    fn new(device: &iced::wgpu::Device, queue: &iced::wgpu::Queue, format: iced::wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}
