pub mod init;

// 文件内的单元测试，#[cfg(test)]表示 只在测试模式下编译
#[cfg(test)]
mod tests {

    #[test]
    fn test_intro() {
        assert_eq!(1, 1);
    }
}
