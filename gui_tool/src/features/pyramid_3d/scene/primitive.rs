use iced::Color;
use iced::Rectangle;
use iced::widget::shader;

use super::camera::Camera;
use super::pipeline::PyramidPipeline;
use super::pipeline::pyramid_shape::PyramidRaw;
use super::pipeline::pyramid_shape::PyramidShape;
use super::pipeline::uniforms::Uniforms;

/// GPU 端图元——持有实例数据和 uniform 参数
///
/// 是 CPU 端 Scene 数据到 GPU 端渲染的桥梁：
/// 1. `new()`: 将 Vec<PyramidShape> 转为 Vec<PyramidRaw>（GPU 友好格式）
/// 2. `prepare()`: 上传数据到 GPU 缓冲区
/// 3. `render()`: 提交 draw call
#[derive(Debug)]
pub struct PyramidPrimitive {
    /// GPU 端实例数据（每个金字塔的变换矩阵 + 颜色）
    pyramid_raws: Vec<PyramidRaw>,
    /// 统一变量（投影矩阵 + 相机位置 + 光照颜色）
    uniforms: Uniforms,
}

impl PyramidPrimitive {
    pub fn new(
        pyramids: &[PyramidShape],
        camera: &Camera,
        bounds: Rectangle,
        light_color: Color,
        world_angle: f32,
        world_scale: f32,
    ) -> PyramidPrimitive {
        Self {
            pyramid_raws: pyramids
                .iter()
                .map(|p| PyramidRaw::from_shape(p, world_angle, world_scale))
                .collect(),
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
        _bounds: &iced::Rectangle,
        viewport: &shader::Viewport,
    ) {
        pipeline.update(
            device,
            queue,
            viewport.physical_size(),
            &self.uniforms,
            self.pyramid_raws.len(),
            &self.pyramid_raws,
        );
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
        pipeline: &Self::Pipeline,
        encoder: &mut iced::wgpu::CommandEncoder,
        target: &iced::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        pipeline.render(
            target,
            encoder,
            *clip_bounds,
            self.pyramid_raws.len() as u32,
        );
    }
}
