// 引用另一个 front_of_house 模块的内容(来自 front_of_house.rs 或 front_of_house 文件夹)
pub mod init;
pub mod mod01;
pub mod netdump;

#[allow(unused)]
fn study_mod01() {
    // 绝对路径
    crate::mod01::hosting::add_to_waitlist(); // 需要如下 pub 权限: pub mod hosting, pub fn add_to_waitlist

    // 相对路径
    mod01::hosting::add_to_waitlist();
    self::mod01::hosting::add_to_waitlist();
}
