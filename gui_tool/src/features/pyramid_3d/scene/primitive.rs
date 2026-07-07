use iced::Color;
use iced::Rectangle;
use iced::widget::shader;

use super::camera::Camera;
use super::pipeline::PyramidPipeline;
use super::pipeline::pyramid_shape::PyramidRaw;
use super::pipeline::pyramid_shape::PyramidShape;
use super::pipeline::uniforms::Uniforms;

#[derive(Debug)]
pub struct PyramidPrimitive {
    pyramid_raws: Vec<PyramidRaw>, // 立方体的GPU友好格式数据（每元素含变换矩阵+法线矩阵）
    uniforms: Uniforms,            // 统一变量（相机投影矩阵+相机位置+光源颜色）
}

impl PyramidPrimitive {
    pub fn new(
        pyramids: Vec<PyramidShape>,
        camera: &Camera,
        bounds: Rectangle,
        light_color: Color,
    ) -> PyramidPrimitive {
        Self {
            pyramid_raws: pyramids
                .iter()
                .map(|x| PyramidRaw::from_shape(x))
                .collect::<Vec<PyramidRaw>>(),
            uniforms: Uniforms::new(camera, bounds, light_color),
        }
    }
}

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
        pipeline.upload(device, &self.pyramid_raws, &self.uniforms);
    }

    fn draw(&self, _pipeline: &Self::Pipeline, _render_pass: &mut iced::wgpu::RenderPass<'_>) -> bool {
        false
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        _encoder: &mut iced::wgpu::CommandEncoder,
        _target: &iced::wgpu::TextureView,
        _clip_bounds: &iced::Rectangle<u32>,
    ) {
        pipeline.render();
    }
}
