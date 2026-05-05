# 文件与 IO

> 磁盘是计算机最慢的组件之一，但也是最可靠的持久化介质——文件 IO 的精髓不在于"如何读写"，而在于该缓冲时缓冲，该同步时同步。

## 1. std::fs 文件操作大全

### 1.1 读取文件

```rust
use std::fs;
use std::io::Read;

fn main() -> std::io::Result<()> {
    // 一次性读取整个文件到字符串（适合小文件）
    let content = fs::read_to_string("Cargo.toml")?;
    println!("文件内容:\n{}", content);

    // 一次性读取为字节向量
    let bytes = fs::read("Cargo.toml")?;
    println!("文件大小: {} 字节", bytes.len());

    // 使用 File 精细控制读取
    let mut file = fs::File::open("Cargo.toml")?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    println!("逐字节读取:\n{}", buffer);

    Ok(())
}
```

> `read_to_string` 是最常用的便捷 API，但它一次性加载全部内容到内存——面对 GB 级日志文件时，请改用 `BufReader` 逐行处理。

### 1.2 写入文件

```rust
use std::fs;
use std::io::Write;

fn main() -> std::io::Result<()> {
    // 一次性写入（创建新文件或截断旧文件）
    fs::write("output.txt", "Hello, World!\n第二行内容")?;

    // 使用 File + Write trait 写入
    let mut file = fs::File::create("output2.txt")?;
    file.write_all(b"byte content\n")?;
    writeln!(file, "通过 writeln! 宏写入")?;

    Ok(())
}
```

### 1.3 文件与目录操作

```rust
use std::fs;

fn main() -> std::io::Result<()> {
    // 创建目录
    fs::create_dir("my_folder")?;
    fs::create_dir_all("deeply/nested/folder")?; // 递归创建

    // 检查存在性
    if fs::metadata("Cargo.toml").is_ok() {
        println!("Cargo.toml 存在");
    }
    // 简写：
    // let exists = Path::new("file.txt").exists();

    // 复制文件
    fs::copy("source.txt", "dest.txt")?;

    // 重命名/移动
    fs::rename("old_name.txt", "new_name.txt")?;

    // 删除文件
    fs::remove_file("temp.txt")?;

    // 删除空目录
    fs::remove_dir("my_folder")?;

    // 删除目录及其内容
    fs::remove_dir_all("deeply")?;

    // 获取元数据
    let meta = fs::metadata("Cargo.toml")?;
    println!("文件大小: {} 字节", meta.len());
    println!("只读: {}", meta.permissions().readonly());
    println!("最后修改: {:?}", meta.modified()?);

    Ok(())
}
```

## 2. OpenOptions：精确控制

```rust
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> std::io::Result<()> {
    // 追加模式
    let mut file = OpenOptions::new()
        .append(true)
        .open("log.txt")?;
    writeln!(file, "追加一行日志")?;

    // 读写模式
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("data.bin")?;

    // create_new：文件必须不存在（原子操作）
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("new_file.txt")?;

    // truncate：打开时清空文件
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("overwrite.txt")?;

    Ok(())
}
```

### 2.1 选项对照表

| 选项 | 含义 |
|------|------|
| `.read(true)` | 允许读取 |
| `.write(true)` | 允许写入 |
| `.append(true)` | 写入定位到文件末尾 |
| `.create(true)` | 文件不存在时创建 |
| `.create_new(true)` | 文件必须不存在，否则报错（与 create 互斥） |
| `.truncate(true)` | 打开时将文件截为零长度（与 append 互斥） |

> OpenOptions 是文件操作的控制面板——正确组合这些开关可以表达几乎所有 POSIX `open()` 标志的语义，而无需逃逸到 FFI。

## 3. BufReader / BufWriter

### 3.1 大文件逐行读取

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    let file = File::open("large_file.txt")?;
    let reader = BufReader::new(file);

    // 逐行读取（内存友好）
    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        println!("第{}行: {}", line_number + 1, line);
        if line_number >= 10 { break; } // 只读前10行
    }

    Ok(())
}
```

### 3.2 BufWriter 缓冲写入

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> std::io::Result<()> {
    let file = File::create("buffered_out.txt")?;
    let mut writer = BufWriter::new(file);

    // 多次写入先在内存缓冲，到达阈值后一次性写入磁盘
    for i in 0..10000 {
        writeln!(writer, "行 {}", i)?;
    }

    // flush 强制将缓冲区写入磁盘
    writer.flush()?;
    Ok(())
}
```

> BufReader/BufWriter 的缓冲区大小默认 8KB——对于顺序读取场景能减少 99% 的系统调用次数，是 IO 性能优化的第一选项。

## 4. Read / Write trait

### 4.1 Read trait 方法

```rust
use std::io::Read;

fn main() -> std::io::Result<()> {
    let mut file = std::fs::File::open("data.txt")?;
    let mut buffer = vec![0u8; 1024];

    // read：尝试读取数据到缓冲区，返回实际读取字节数
    let n = file.read(&mut buffer)?;
    println!("读取了 {} 字节", n);

    // read_to_end：读取到 Vec<u8> 末尾
    let mut all_data = Vec::new();
    file.read_to_end(&mut all_data)?;

    // read_exact：精确读取指定数量字节（不足则 Err）
    let mut exact_buffer = [0u8; 16];
    file.read_exact(&mut exact_buffer)?;

    // read_to_string：读取到 String
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // bytes()：返回字节迭代器
    for byte_result in file.bytes() {
        let byte = byte_result?;
        // 处理每个字节
    }

    Ok(())
}
```

### 4.2 Write trait 方法

```rust
use std::io::Write;

fn main() -> std::io::Result<()> {
    let mut file = std::fs::File::create("output.txt")?;

    // write：写入字节切片
    file.write(b"Hello")?;

    // write_all：确保所有数据写入完毕
    file.write_all(b"World")?;

    // flush：刷新缓冲区到磁盘
    file.flush()?;

    // 使用 writeln! 宏（需要 use std::io::Write）
    writeln!(file, "格式化输出: {}", 42)?;

    Ok(())
}
```

> Read/Write 是 Rust IO 生态的两大核心 trait——实现了它们，你就自动获得了 BufReader、链式操作、多种适配器的能力。

## 5. Seek 定位

```rust
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

fn main() -> std::io::Result<()> {
    let mut file = File::create("seek_demo.bin")?;
    file.write_all(b"0123456789ABCDEF")?;

    let mut file = File::open("seek_demo.bin")?;
    let mut buf = [0u8; 4];

    // 从开头偏移
    file.seek(SeekFrom::Start(10))?;
    file.read_exact(&mut buf)?;
    println!("Start(10): {:?}", std::str::from_utf8(&buf)); // "ABCD"

    // 从末尾回退
    file.seek(SeekFrom::End(-3))?;
    file.read_exact(&mut buf)?;
    println!("End(-3): {:?}", std::str::from_utf8(&buf)); // "DEF\0"

    // 从当前位置偏移
    file.seek(SeekFrom::Current(3))?;
    file.read_exact(&mut buf)?;

    // 获取当前位置
    let pos = file.seek(SeekFrom::Current(0))?;
    println!("当前位置: {}", pos);

    Ok(())
}
```

| SeekFrom 变体 | 含义 |
|---------------|------|
| `Start(n)` | 从文件开头偏移 n 字节 |
| `End(n)` | 从文件末尾偏移 n 字节（负值向前） |
| `Current(n)` | 从当前位置偏移 n 字节 |

## 6. Cursor：内存中的 IO

```rust
use std::io::{Cursor, Read, Write, Seek, SeekFrom};

fn main() -> std::io::Result<()> {
    // Cursor 是对内存切片/向量的读写封装
    let mut cursor = Cursor::new(Vec::new());

    // 写入到内存
    cursor.write_all(b"Hello, Cursor!")?;
    writeln!(cursor, "\n第二行")?;

    // 回溯读取
    cursor.seek(SeekFrom::Start(0))?;
    let mut content = String::new();
    cursor.read_to_string(&mut content)?;
    println!("Cursor 内容:\n{}", content);

    // 初始化时提供数据
    let data = b"initial data";
    let mut cursor = Cursor::new(data);
    let mut buf = [0u8; 7];
    cursor.read_exact(&mut buf)?;
    println!("{:?}", std::str::from_utf8(&buf));

    // 获取内部数据
    let inner = cursor.into_inner();
    println!("原始字节: {:?}", inner);

    Ok(())
}
```

> Cursor 是测试 IO 逻辑的利器——你不需要真的创建文件，就能验证 Read/Write/Seek 实现的正确性。

## 7. stdin / stdout / stderr

```rust
use std::io::{self, Read, Write, BufRead};

fn main() -> io::Result<()> {
    // 标准输入
    let mut stdin = io::stdin();
    let mut input = String::new();
    print!("请输入: ");
    io::stdout().flush()?; // 确保提示先显示

    // 按行读取
    let mut handle = io::stdin().lock(); // 锁定以获得更快性能
    handle.read_line(&mut input)?;
    println!("你输入了: {}", input.trim());

    // 标准输出（被行缓冲）
    io::stdout().write_all(b"写入标准输出\n")?;

    // 标准错误（不被缓冲）
    io::stderr().write_all(b"错误信息写入 stderr\n")?;

    // 锁定以获得缓冲性能
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "快速写入")?;

    Ok(())
}
```

## 8. 目录遍历

### 8.1 std::fs::read_dir

```rust
use std::fs;

fn main() -> std::io::Result<()> {
    let entries = fs::read_dir(".")?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        let type_str = if metadata.is_dir() { "目录" }
                      else if metadata.is_file() { "文件" }
                      else { "其他" };

        println!("{} ({})", path.display(), type_str);
    }

    Ok(())
}
```

### 8.2 walkdir crate 递归遍历

```rust
// Cargo.toml: walkdir = "2"
use walkdir::WalkDir;

fn main() {
    for entry in WalkDir::new(".")
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        println!("{}", entry.path().display());
    }
}
```

> read_dir 适合单层列出，WalkDir 适合递归爬取——两者互补，覆盖了从"看一眼"到"地毯式搜索"的全部目录遍历场景。

## 9. Path / PathBuf

```rust
use std::path::{Path, PathBuf};

fn main() {
    // 创建路径
    let path = Path::new("/usr/local/bin/rustc");
    let mut buf = PathBuf::from("/usr");
    buf.push("local");
    buf.push("bin");
    buf.push("rustc");
    assert_eq!(path, buf);

    // 路径操作
    println!("父目录: {:?}", path.parent());
    println!("文件名: {:?}", path.file_name());
    println!("文件主名: {:?}", path.file_stem());
    println!("扩展名: {:?}", path.extension());
    println!("存在: {}", path.exists());
    println!("是绝对路径: {}", path.is_absolute());

    // 组件迭代
    for component in path.components() {
        println!("组件: {:?}", component);
    }

    // 连接路径
    let home = Path::new("/home/user");
    let config = home.join(".config").join("app");
    println!("配置路径: {}", config.display());

    // 读取父/祖先路径
    let path = Path::new("/a/b/c/d.txt");
    println!("路径前两段祖先: {:?}", path.ancestors().nth(2));
}
```

## 10. serde 文件序列化

```rust
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    name: String,
    version: String,
    port: u16,
    features: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        name: "my-app".into(),
        version: "1.0.0".into(),
        port: 8080,
        features: vec!["auth".into(), "logging".into()],
    };

    // 序列化到 JSON 文件
    let json = serde_json::to_string_pretty(&config)?;
    fs::write("config.json", json)?;

    // TOML 格式
    let toml_str = toml::to_string(&config)?;
    fs::write("config.toml", toml_str)?;

    // 从文件反序列化
    let content = fs::read_to_string("config.json")?;
    let loaded: Config = serde_json::from_str(&content)?;
    assert_eq!(config, loaded);

    Ok(())
}
```

> serde + 文件读写的组合是 Rust 持久化的黄金配方——类型安全 + 零成本抽象，让你在编译期就知道数据格式是否正确。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| `read_to_string` 读取二进制文件 | 非 UTF-8 字节无法转换成 String 导致 panic | 二进制文件用 `read` 读入 `Vec<u8>` |
| 打开文件后忘记关闭 | Rust 的 RAII 自动 Drop 关闭，但错误可能被忽略 | 对关键文件操作检查返回的 `Result`；需要时手动 flush |
| BufWriter 忘记 flush | 缓冲区未满时数据不会写入磁盘 | 关键数据在操作完成后立即 flush |
| seek 偏移用正数对 End 定位 | End 的正偏移越过了文件末尾 | End 偏移应为 0 或负数 |
| Path::new 不检查文件是否存在 | `Path` 是纯字符串表示 | 使用 `path.exists()` 或 `metadata()` |
| `write(true)` + `create_new(true)` 组合 | 文件存在时 `create_new` 会报错 | 确认组合的语义：create_new 要求文件必须不存在 |
| `remove_dir` vs `remove_dir_all` 用错 | remove_dir 只在目录为空时成功 | 删除非空目录使用 remove_dir_all |
| 在 Windows 和 Unix 间路径分隔符不同 | `\\` vs `/` | 使用 Path::join 和 Path::components 而非字符串拼接 |

> **详见测试**: `tests/rust_features/26_file_and_io.rs`
