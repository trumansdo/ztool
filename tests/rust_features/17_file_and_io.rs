// ---------------------------------------------------------------------------
// 4.6 文件与 IO (1.87+ / 1.89+)
// ---------------------------------------------------------------------------

#[test]
/// 测试: 匿名管道 std::io::pipe() (1.87+)
fn test_anonymous_pipe() {
    // 语法: std::io::pipe() 创建匿名管道, 返回 (Reader, Writer) (1.87+)
    // 避坑: Writer 必须 drop 或 close 才能让 Reader 读到 EOF; 管道有缓冲区大小限制
    use std::io::{Read, Write};
    let (mut reader, mut writer) = std::io::pipe().expect("创建管道失败");
    writer
        .write_all(b"hello pipe")
        .expect("写入失败");
    drop(writer);
    let mut buf = Vec::new();
    reader
        .read_to_end(&mut buf)
        .expect("读取失败");
    assert_eq!(buf, b"hello pipe");
}

#[test]
/// 测试: File::lock() 文件独占锁 (1.89+)
fn test_file_lock() {
    // 语法: File::lock() 获取独占锁, File::try_lock() 非阻塞尝试 (1.89+)
    // 避坑: 锁在 File drop 时自动释放; 不同平台实现不同(flock/fcntl/LockFileEx)
    use std::fs::File;
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("rust_test_lock.txt");
    let file = File::create(&path).expect("创建文件失败");
    file.lock().expect("获取锁失败");
    drop(file);
    let _ = std::fs::remove_file(&path);
}

#[test]
/// 测试: File 读写基础操作
fn test_file_read_write() {
    // 语法: std::fs::read/write 一次性读取/写入整个文件
    // 避坑: read 将整个文件加载到内存, 大文件用 BufReader; write 会覆盖已有文件
    use std::fs;

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("rust_test_io.txt");

    // 写入
    fs::write(&path, b"hello file").expect("写入失败");

    // 读取
    let content = fs::read(&path).expect("读取失败");
    assert_eq!(content, b"hello file");

    // 读取为字符串
    let text = fs::read_to_string(&path).expect("读取失败");
    assert_eq!(text, "hello file");

    let _ = fs::remove_file(&path);
}

#[test]
/// 测试: BufReader / BufWriter 缓冲 IO
fn test_buffered_io() {
    // 语法: BufReader/BufWriter 提供缓冲, 减少系统调用次数
    // 避坑: BufWriter 必须 flush() 或 drop 才能确保数据写入; 小文件不需要缓冲
    use std::fs::File;
    use std::io::{BufRead, BufReader, BufWriter, Write};

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("rust_test_bufio.txt");

    // 写入
    let file = File::create(&path).expect("创建失败");
    let mut writer = BufWriter::new(file);
    writer
        .write_all(b"line1\nline2\nline3\n")
        .expect("写入失败");
    writer
        .flush()
        .expect("flush 失败");
    drop(writer);

    // 读取
    let file = File::open(&path).expect("打开失败");
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .map(|l| l.unwrap())
        .collect();
    assert_eq!(lines, vec!["line1", "line2", "line3"]);

    let _ = std::fs::remove_file(&path);
}

#[test]
/// 测试: Seek 文件定位
fn test_file_seek() {
    // 语法: Seek trait 提供 seek() 方法, 支持 SeekFrom::Start/Current/End
    // 避坑: seek 操作的是字节偏移, 不是行号; 对 stdin/stdout seek 可能失败
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom, Write};

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("rust_test_seek.txt");

    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .expect("打开失败");

    file.write_all(b"hello world")
        .expect("写入失败");

    // 定位到开头
    file.seek(SeekFrom::Start(0))
        .expect("seek 失败");
    let mut buf = [0u8; 5];
    file.read_exact(&mut buf)
        .expect("读取失败");
    assert_eq!(&buf, b"hello");

    // 从当前位置偏移
    file.seek(SeekFrom::Current(1))
        .expect("seek 失败");
    file.read_exact(&mut buf)
        .expect("读取失败");
    assert_eq!(&buf, b"world");

    let _ = std::fs::remove_file(&path);
}

#[test]
/// 测试: Stdin / Stdout / Stderr 标准流
fn test_standard_streams() {
    // 语法: std::io::stdin()/stdout()/stderr() 获取标准流
    // 避坑: stdin 是行缓冲的; stdout 在终端时行缓冲, 重定向时全缓冲
    //       stderr 默认无缓冲
    use std::io::{self, Write};

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(b"test stdout")
        .expect("写入失败");
    handle
        .flush()
        .expect("flush 失败");
}

#[test]
/// 测试: Cursor 内存中的 Seek
fn test_cursor() {
    // 语法: Cursor<T> 将 Vec/ &[u8] 包装为可 seek 的 Read/Write
    // 避坑: Cursor 不涉及文件系统, 纯内存操作; 适合测试和协议解析
    use std::io::{Cursor, Read, Seek, SeekFrom, Write};

    let mut cursor = Cursor::new(Vec::new());
    cursor
        .write_all(b"hello world")
        .expect("写入失败");

    cursor
        .seek(SeekFrom::Start(6))
        .expect("seek 失败");
    cursor
        .write_all(b"Rust")
        .expect("写入失败");

    let result = cursor.into_inner();
    assert_eq!(&result, b"hello Rustd");
}

#[test]
/// 测试: File::options() 构建器模式
fn test_file_options() {
    // 语法: File::options() 提供链式 API 配置打开模式
    // 避坑: 必须至少指定一个打开模式 (read/write/create/append/truncate)
    use std::fs::File;

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("rust_test_options.txt");

    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .expect("打开失败");

    assert!(file.metadata().is_ok());
    let _ = std::fs::remove_file(&path);
}
