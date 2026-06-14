use iced::widget::shader;

use super::pipeline::PyramidPipeline;

#[derive(Debug)]
pub struct PyramidPrimitive {}

impl shader::Primitive for PyramidPrimitive {
    type Pipeline = PyramidPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &iced::wgpu::Device,
        queue: &iced::wgpu::Queue,
        bounds: &iced::Rectangle,
        viewport: &shader::Viewport,
    ) {
        todo!()
    }
    
    fn draw(
        &self,
        _pipeline: &Self::Pipeline,
        _render_pass: &mut iced::wgpu::RenderPass<'_>,
    ) -> bool {
        false
    }
    
    fn render(
        &self,
        _pipeline: &Self::Pipeline,
        _encoder: &mut iced::wgpu::CommandEncoder,
        _target: &iced::wgpu::TextureView,
        _clip_bounds: &iced::Rectangle<u32>,
    ) {
    }

    
}
