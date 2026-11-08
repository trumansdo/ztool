use std::time::Instant;

use glam::{self, Mat4, Quat, Vec3};

#[test]
pub fn test01() {
    let proj = glam::Mat4::perspective_rh(45f32, 1000f32 / 500f32, 0.1f32, 100f32);
    // println!("{}", proj);

    // println!("{}", std::f64::consts::PI);
    // println!("{}", glam::Quat::IDENTITY);
    let quat = Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI);
    println!("{}", quat);
    let rotation_matrix = Mat4::from_quat(quat); // 转成矩阵给 shader
    println!("{}", rotation_matrix);
}

#[test]
pub fn test02() {
    println!("{:?}", Instant::now().elapsed());
}
