# 17 - 文件与 IO 操作

## 概述

Rust 的 IO 库提供了安全且高效的文件和网络操作。与其他语言不同，Rust 的 IO 操作是同步的，但可以通过 async 运行时转换为异步。

## 文件操作

```rust
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};

// 创建/打开
let mut file = File::create("output.txt")?;
let mut file = File::open("input.txt")?;

// 读取
let mut content = String::new();
file.read_to_string(&mut content)?;

// 写入
file.write_all(b"hello")?;
file.flush()?;
```

## 文件选项

```rust
let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .append(true)
    .open("file.txt")?;
```

## BufReader / BufWriter

缓冲 IO 提高性能：

```rust
use std::io::{BufReader, BufWriter};

let file = File::open("input.txt")?;
let reader = BufReader::new(file);

let mut line = String::new();
reader.read_line(&mut line)?;

let output = File::create("output.txt")?;
let mut writer = BufWriter::new(output);
writer.write_all(b"data")?;
```

## Seek

```rust
use std::io::{Seek, SeekFrom};

// 从开头
file.seek(SeekFrom::Start(0))?;

// 从结尾
file.seek(SeekFrom::End(-10))?;

// 从当前位置
file.seek(SeekFrom::Current(5))?;
```

## Cursor

内存中的 IO：

```rust
use std::io::{Cursor, Read};

let mut cursor = Cursor::new(vec![1, 2, 3, 4, 5]);
let mut buf = [0u8; 2];
cursor.read(&mut buf)?; // buf = [1, 2]
cursor.seek(SeekFrom::Start(0))?;
```

## 路径操作

```rust
use std::path::Path;

let path = Path::new("dir/file.txt");
path.exists();
path.is_file();
path.is_dir();

let parent = path.parent();
let file_name = path.file_name();
let extension = path.extension();
```

## 目录操作

```rust
use std::fs;

// 读取目录
for entry in fs::read_dir(".")? {
    let entry = entry?;
    println!("{}", entry.file_name());
}

// 创建目录
fs::create_dir("new_dir")?;
fs::create_dir_all("path/to/dir")?;

// 删除
fs::remove_file("file.txt")?;
fs::remove_dir("dir")?;
```

## 单元测试

详见 `tests/rust_features/17_file_and_io.rs`

## 参考资料

- [Rust IO Docs](https://doc.rust-lang.org/std/io/index.html)
- [Rust fs module](https://doc.rust-lang.org/std/fs/)