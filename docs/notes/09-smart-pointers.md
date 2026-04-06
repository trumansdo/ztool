# 09 - 智能指针

## 核心概念

智能指针拥有数据并提供额外功能：

### Box<T>

堆分配:

```rust
let boxed = Box::new(42);
let value = *boxed;
```

递归类型:

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

### Rc<T>

引用计数(单线程):

```rust
let rc = Rc::new(vec![1, 2]);
let clone = Rc::clone(&rc);
println!("{}", Rc::strong_count(&rc));
```

### Arc<T>

原子引用计数(多线程):

```rust
use std::sync::Arc;
let arc = Arc::new(data);
```

### 内部可变性

#### Cell<T>

```rust
use std::cell::Cell;
let cell = Cell::new(10);
cell.set(20);
```

#### RefCell<T>

```rust
use std::cell::RefCell;
let refcell = RefCell::new(vec![1,2,3]);
let mut borrow = refcell.borrow_mut();
borrow.push(4);
```

## Cow<T>

Copy-on-Write，惰性克隆:

```rust
use std::borrow::Cow;
let text = Cow::Borrowed("hello");
let owned = text.to_mut();  // 转为_owned
```

## 单元测试

详见 `tests/rust_features/09_smart_pointers.rs`
