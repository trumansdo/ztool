pub mod buffer;
pub mod pyramid_shape;
pub mod uniforms;
pub mod vertex;

use iced::widget::shader::Pipeline;
use iced::{wgpu, widget::shader};

use pyramid_shape::PyramidRaw;
use uniforms::Uniforms;

#[derive(Debug)]
pub struct PyramidPipeline {}

impl Pipeline for PyramidPipeline {
    fn new(device: &iced::wgpu::Device, queue: &iced::wgpu::Queue, format: iced::wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

impl PyramidPipeline {
    pub fn upload(&mut self,device: &iced::wgpu::Device,  pyramid_raws: &Vec<PyramidRaw>, uniforms: &Uniforms) {
        
    }

    pub fn render(&self) {}
}
