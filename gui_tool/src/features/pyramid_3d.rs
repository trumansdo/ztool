/// 3D 金字塔模块——基于 Iced + wgpu 的自定义着色器示例
///
/// 架构层次（自顶向下）：
/// - view.rs:   UI 布局（shader widget + 控制面板）
/// - update.rs: 消息处理、动画更新、Pyramid 结构体
/// - scene.rs:  3D 场景（实现 shader::Program trait）
///   - camera.rs:    相机控制（MVP 矩阵中的 VP）
///   - primitive.rs: GPU 图元（实现 shader::Primitive trait）
///   - pipeline.rs:  GPU 管线（实现 shader::Pipeline trait）
///     - vertex.rs:         顶点数据结构（位置 + 法线）
///     - uniforms.rs:       统一变量（投影矩阵 + 相机 + 光照）
///     - pyramid_shape.rs:  金字塔几何 + 实例数据
///     - buffer.rs:         GPU 缓冲区封装（动态扩容）

mod scene;
mod update;
mod view;

pub use update::{Msg, Pyramid};
pub use view::view;
pub use update::update;
