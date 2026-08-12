use iced::Color;
use iced::Rectangle;
use iced::widget::shader;

use super::camera::Camera;
use super::pipeline::PyramidPipeline;
use super::pipeline::pyramid_shape::PyramidRaw;
use super::pipeline::pyramid_shape::PyramidShape;
use super::pipeline::uniforms::Uniforms;

/// CPU 端图元：封装每帧变化的渲染数据
///
/// 持有实例数据（变换矩阵+颜色）和统一变量（VP矩阵+相机+光源），
/// 在 prepare() 中上传到 GPU，在 render() 中执行离屏绘制。
#[derive(Debug)]
pub struct PyramidPrimitive {
    pyramid_raws: Vec<PyramidRaw>,
    uniforms: Uniforms,
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
        // 上传实例数据和 Uniform 数据到 GPU
        pipeline.upload(device, queue, &self.pyramid_raws, &self.uniforms);

        // 存储 target 纹理的真实尺寸（viewport.physical_size 与 Iced 分配的 target 纹理一致）
        let physical = viewport.physical_size();
        pipeline.set_target_size(physical.width, physical.height);

        // 首帧初始化深度纹理
        pipeline.ensure_depth_texture(physical.width, physical.height);
    }

    fn draw(&self, _pipeline: &Self::Pipeline, _render_pass: &mut iced::wgpu::RenderPass<'_>) -> bool {
        // 返回 false，使用自定义离屏 RenderPass
        false
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut iced::wgpu::CommandEncoder,
        target: &iced::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        pipeline.render(encoder, target, clip_bounds);
    }
}
