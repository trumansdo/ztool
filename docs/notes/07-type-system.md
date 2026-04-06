# 07 - 类型系统与 Trait

## 核心概念

### Trait 定义

```rust
trait MyTrait {
    fn method(&self) -> i32;  // 方法签名
    
    fn default_method(&self) -> i32 {
        42  // 默认实现
    }
}
```

### Trait 实现

```rust
struct MyType;
impl MyTrait for MyType {
    fn method(&self) -> i32 { 10 }
}
```

### Trait 对象 dyn Trait

```rust
let obj: &dyn MyTrait = &MyType;
let boxed: Box<dyn MyTrait> = Box::new(MyType);
```

## 对象安全

Trait 要能作为 dyn Trait 使用，必须满足：

- 不能有泛型方法
- 不能返回 Self
- 不能有 `self: Sized` 约束

可用 `where Self: Sized` 标记非对象安全方法。

## 关联类型

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

## 上转casting

Rust 1.86+ 支持子 trait 对象自动转为超 trait 对象：

```rust
trait Super { fn method(&self) -> i32; }
trait Sub: Super { fn sub_method(&self) -> i32; }

let sub: &dyn Sub = &MyStruct;
let super_ref: &dyn Super = sub;  // 隐式 upcast
```

## 单元测试

详见 `tests/rust_features/07_type_system.rs`
