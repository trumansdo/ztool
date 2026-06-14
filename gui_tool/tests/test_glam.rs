use glam::{self, vec3};
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    // 顶点在模型空间(局部坐标系)中的三维坐标位置 (x, y, z)
    pub pos: glam::Vec3,
    // 表面法线方向，归一化向量，指向面外侧，用于漫反射/镜面反射等光照计算
    // pub normal: glam::Vec3,
    // // 切线向量(Tangent)，沿纹理UV的U方向，与法线、副法线构成切线空间(TBN矩阵)
    // // 用于法线贴图(Normal Mapping): 将切线空间法线变换到世界空间以正确计算光照
    // pub tangent: glam::Vec3,
    // // 纹理坐标(UV)，将2D纹理映射到模型表面
    // // (0,0) 表示纹理左下角，(1,1) 表示纹理右上角
    // pub uv: glam::Vec2,
}
#[test]
fn demo() {
    let x = glam::Mat3A::from_rotation_z(0f32);
    println!("{}", x);
    let y = glam::Mat4::look_at_rh(vec3(0.0, 10.0, 10.0), glam::Vec3::ZERO, glam::Vec3::Y);
    println!("{}", y);
    let z = glam::Mat4::look_to_rh(vec3(0.0, 10.0, 10.0), glam::Vec3::ZERO, glam::Vec3::Y);
    println!("{}", z);
    println!("{}", std::mem::size_of::<glam::Vec3>());
    println!("{}", std::mem::size_of::<Vertex>());
}
