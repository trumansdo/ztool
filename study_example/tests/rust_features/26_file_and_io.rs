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
    use std::io::{Cursor, Seek, SeekFrom, Write};

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

// ===========================================================================
// 补充增强测试
// ===========================================================================

#[test]
/// 测试: std::fs 文件操作大全(create/read/write/remove/rename/copy)
fn test_fs_operations_catalog() {
    // 语法: std::fs 提供完整的文件操作 API
    // 避坑: 所有操作都返回 Result，必须处理错误；rename 跨文件系统可能失败
    use std::fs;

    let temp_dir = std::env::temp_dir();
    let base = temp_dir.join("rust_fs_catalog");

    // 创建目录
    fs::create_dir_all(&base).expect("创建目录失败");

    // 创建并写入文件
    let file_path = base.join("data.txt");
    fs::write(&file_path, b"Hello, Rust FS!").expect("写入失败");

    // 读取文件
    let content = fs::read_to_string(&file_path).expect("读取失败");
    assert_eq!(content, "Hello, Rust FS!");

    // 元数据
    let meta = fs::metadata(&file_path).expect("获取元数据失败");
    assert!(meta.is_file());
    assert!(meta.len() > 0);

    // 拷贝文件
    let copy_path = base.join("copy.txt");
    fs::copy(&file_path, &copy_path).expect("拷贝失败");
    assert!(copy_path.exists());

    // 重命名
    let rename_path = base.join("renamed.txt");
    fs::rename(&copy_path, &rename_path).expect("重命名失败");
    assert!(!copy_path.exists());
    assert!(rename_path.exists());

    // 删除文件
    fs::remove_file(&rename_path).expect("删除文件失败");

    // 删除目录
    fs::remove_dir_all(&base).expect("删除目录失败");
    assert!(!base.exists());
}

#[test]
/// 测试: OpenOptions 详细用法(create_new 原子创建)
fn test_open_options_detailed() {
    // 语法: OpenOptions 的 create_new(true) 实现原子创建，避免 TOCTOU 竞态
    // 避坑: create_new 与 create 互斥，且 append 与 truncate 不应同时使用
    use std::fs::OpenOptions;
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("rust_openoptions.txt");

    // 使用 OpenOptions::new() 传统写法
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .expect("OpenOptions 打开失败");
        file.write_all(b"OpenOptions write").expect("写入失败");
    }

    // create_new: 文件已存在则报错
    let result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path);
    assert!(result.is_err());

    // append 模式：所有写入都追加到末尾
    {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("append 打开失败");
        file.write_all(b" + append").expect("追加写入失败");
    }
    let content = std::fs::read_to_string(&path).expect("读取失败");
    assert!(content.contains("append"));

    let _ = std::fs::remove_file(&path);
}

#[test]
/// 测试: 大文件逐行读取 BufReader::lines()
fn test_large_file_line_reading() {
    // 语法: BufReader::lines() 返回行迭代器，惰性读取不爆内存
    // 避坑: lines() 迭代器去掉换行符；read_line() 保留换行符
    use std::fs::File;
    use std::io::{BufRead, BufReader, Write};

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("rust_lines.txt");

    // 生成多行数据
    {
        let mut file = File::create(&path).expect("创建失败");
        for i in 0..100 {
            writeln!(file, "第 {} 行", i).expect("写入失败");
        }
    }

    // 方式1: lines() 迭代器（去掉了换行符）
    let file = File::open(&path).expect("打开失败");
    let reader = BufReader::new(file);
    let count = reader.lines().count();
    assert_eq!(count, 100);

    // 方式2: 手动 read_line（保留换行符，复用缓冲区）
    let file = File::open(&path).expect("打开失败");
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut line_count = 0;
    while reader.read_line(&mut line).expect("读取失败") > 0 {
        assert!(line.ends_with('\n'));
        line.clear();
        line_count += 1;
    }
    assert_eq!(line_count, 100);

    let _ = std::fs::remove_file(&path);
}

#[test]
/// 测试: Read/Write trait 与 &[u8] 的转换
fn test_read_write_trait_conversions() {
    // 语法: &[u8] 实现了 Read，Vec<u8> 实现了 Write —— 字节切片可直接当"文件"用
    // 避坑: &[u8] 的 Read 实现只读到切片末尾，不会增长
    use std::io::{Read, Write};

    // &[u8] 实现 Read —— 从字节切片读取
    let data: &[u8] = b"hello from slice";
    let mut reader: &[u8] = data; // &[u8] 本身实现了 Read
    let mut buf = String::new();
    reader.read_to_string(&mut buf).expect("读取失败");
    assert_eq!(buf, "hello from slice");

    // Vec<u8> 实现 Write —— 写入到动态数组
    let mut writer: Vec<u8> = Vec::new();
    writer.write_all(b"write to vec").expect("写入失败");
    assert_eq!(writer, b"write to vec");

    // std::io::copy: 通用的流拷贝
    let mut source: &[u8] = b"source_data";
    let mut dest: Vec<u8> = Vec::new();
    let bytes = std::io::copy(&mut source, &mut dest).expect("拷贝失败");
    assert_eq!(bytes, 11);
    assert_eq!(dest, b"source_data");
}

#[test]
/// 测试: 目录遍历 read_dir
fn test_directory_traversal() {
    // 语法: std::fs::read_dir 遍历单级目录
    // 避坑: read_dir 不递归；遍历时需注意 entry 可能因权限等问题为 Err
    use std::fs;

    let temp_dir = std::env::temp_dir();
    let base = temp_dir.join("rust_readdir_test");

    fs::create_dir_all(&base).expect("创建目录失败");
    fs::write(base.join("a.txt"), b"a").expect("写入失败");
    fs::write(base.join("b.txt"), b"b").expect("写入失败");
    fs::create_dir(base.join("sub")).expect("创建子目录失败");

    // 单级遍历
    let entries: Vec<_> = fs::read_dir(&base)
        .expect("读取目录失败")
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(entries.len(), 3); // a.txt, b.txt, sub

    // 按类型筛选
    let files: Vec<_> = entries
        .iter()
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .collect();
    assert_eq!(files.len(), 2); // a.txt, b.txt

    let dirs: Vec<_> = entries
        .iter()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    assert_eq!(dirs.len(), 1); // sub

    fs::remove_dir_all(&base).expect("清理失败");
}

#[test]
/// 测试: 临时文件 tempfile
fn test_tempfile_usage() {
    // 语法: tempfile crate 提供自动清理的临时文件和目录
    // 避坑: NamedTempFile drop 时自动删除，需 persist 才能保留
    use std::io::Write;

    // 创建有名临时文件
    let mut tmp = tempfile::NamedTempFile::new().expect("创建临时文件失败");
    let tmp_path = tmp.path().to_path_buf();

    write!(tmp, "临时数据").expect("写入失败");
    tmp.flush().expect("flush 失败");

    // 验证内容
    let content = std::fs::read_to_string(&tmp_path).expect("读取失败");
    assert_eq!(content, "临时数据");

    // persist: 持久化保留，防止 drop 时删除
    let perm_path = std::env::temp_dir().join("rust_test_persisted.txt");
    tmp.persist(&perm_path).expect("persist 失败");
    assert!(perm_path.exists());

    // 清理
    let _ = std::fs::remove_file(&perm_path);
}

#[test]
/// 测试: Path/PathBuf 操作(join, extension, file_name, parent)
fn test_path_operations() {
    // 语法: Path 是不可变路径视图，PathBuf 是可变的路径拥有者
    // 避坑: 跨平台路径拼接用 Path::join() 而非字符串拼接
    use std::path::{Path, PathBuf};

    // join: 拼接路径
    let base = Path::new("/home/user");
    let full = base.join("docs").join("readme.txt");
    assert!(
        full.to_str()
            .unwrap()
            .contains("docs")
            && full.to_str().unwrap().contains("readme.txt")
    );

    // 路径分解
    let path = Path::new("/home/user/file.txt");
    assert_eq!(path.parent(), Some(Path::new("/home/user")));
    assert_eq!(path.file_name().unwrap(), "file.txt");
    assert_eq!(path.file_stem().unwrap(), "file");
    assert_eq!(path.extension().unwrap(), "txt");

    // with_extension: 修改扩展名
    let new = Path::new("config.json").with_extension("yaml");
    assert_eq!(new.file_name().unwrap(), "config.yaml");

    // with_file_name: 修改文件名
    let new = Path::new("/home/old.txt").with_file_name("new.txt");
    assert_eq!(new.file_name().unwrap(), "new.txt");

    // &Path <-> PathBuf 互转
    let buf_path: PathBuf = Path::new("hello").to_path_buf();
    let borrowed: &Path = buf_path.as_path();
    assert_eq!(borrowed.to_str().unwrap(), "hello");
}

#[test]
/// 测试: 序列化到文件(serde_json + serde_yml)
fn test_serialize_to_file() {
    // 语法: serde_json/serde_yml + std::fs 实现结构体到文件的持久化
    // 避坑: 反序列化时必须处理格式不匹配的错误
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        port: u16,
    }

    let config = Config {
        name: "测试服务".to_string(),
        port: 8080,
    };

    let temp_dir = std::env::temp_dir();

    // JSON 序列化到文件
    let json_path = temp_dir.join("rust_test_cfg.json");
    let json_str = serde_json::to_string_pretty(&config).expect("JSON序列化失败");
    std::fs::write(&json_path, &json_str).expect("写入JSON文件失败");

    let loaded: Config =
        serde_json::from_str(&std::fs::read_to_string(&json_path).expect("读取JSON失败"))
            .expect("JSON反序列化失败");
    assert_eq!(loaded, config);

    // YAML 序列化到文件
    let yaml_path = temp_dir.join("rust_test_cfg.yaml");
    let yaml_str = serde_yml::to_string(&config).expect("YAML序列化失败");
    std::fs::write(&yaml_path, &yaml_str).expect("写入YAML文件失败");

    let loaded: Config =
        serde_yml::from_str(&std::fs::read_to_string(&yaml_path).expect("读取YAML失败"))
            .expect("YAML反序列化失败");
    assert_eq!(loaded, config);

    // 清理
    let _ = std::fs::remove_file(&json_path);
    let _ = std::fs::remove_file(&yaml_path);
}

#[test]
/// 测试: Cursor 用于协议解析(模拟二进制协议头解析)
fn test_cursor_protocol_parsing() {
    // 语法: Cursor 可在内存中模拟网络数据包解析
    // 避坑: Cursor 的 Seek 和 Read 是独立的，seek 后 read 从新位置开始
    use std::io::{Cursor, Read};

    // 模拟协议头: [版本(1byte)] [长度(2bytes,大端)] [负载(N bytes)]
    let header: Vec<u8> = vec![
        0x02,             // 版本 2
        0x00, 0x0B,       // 长度 11 (大端序)
        b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd',
    ];

    let mut cursor = Cursor::new(header);

    // 读版本
    let mut version = [0u8; 1];
    cursor.read_exact(&mut version).expect("读版本失败");
    assert_eq!(version[0], 0x02);

    // 读长度(大端序)
    let mut len_bytes = [0u8; 2];
    cursor.read_exact(&mut len_bytes).expect("读长度失败");
    let payload_len = u16::from_be_bytes(len_bytes);
    assert_eq!(payload_len, 11);

    // 读负载
    let mut payload = vec![0u8; payload_len as usize];
    cursor.read_exact(&mut payload).expect("读负载失败");
    assert_eq!(payload, b"Hello World");
}
