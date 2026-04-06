# 17 - 文件与 IO 操作

## 核心概念

### 文件操作

```rust
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

// 创建/打开
let mut file = File::create("output.txt")?;
let mut file = File::open("input.txt")?;

// 读取
let mut content = String::new();
file.read_to_string(&mut content)?;

// 写入
file.write_all(b"hello")?;
```

### 文件选项

```rust
let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .append(true)
    .open("file.txt")?;
```

### BufReader / BufWriter

缓冲 IO:

```rust
use std::io::{BufReader, BufWriter};
let reader = BufReader::new(file);
let writer = BufWriter::new(file);
```

### Seek

```rust
use std::io::{Seek, SeekFrom};
file.seek(SeekFrom::Start(0))?;
file.seek(SeekFrom::End(-10))?;
```

### Cursor

内存中的 IO:

```rust
use std::io::{Cursor, Read};
let mut cursor = Cursor::new(vec![1, 2, 3]);
let mut buf = [0u8; 2];
cursor.read(&mut buf)?;
```

## 单元测试

详见 `tests/rust_features/17_file_and_io.rs`
