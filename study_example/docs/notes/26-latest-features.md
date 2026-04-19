# 26 - 最新特性快速参考

## 核心概念

### 裸函数 naked functions (1.88+)

```rust
#[unsafe(naked)]
fn bare_metal() {
    // 完全控制汇编
}
```

### 安全架构 intrinsic (1.87+)

启用 target_feature 后，原 unsafe 的 intrinsic 变为安全：

```rust
if is_x86_feature_detected!("sse2") {
    // SSE2 可用
}
```

### asm! 标签操作数 (1.87+)

内联汇编支持 label 跳转。

### cargo publish --workspace (1.90+)

一次性发布工作区所有包。

### LLD 链接器 (1.90+)

Linux x86_64 默认使用 LLD，链接速度大幅提升。

### const 泛型

```rust
fn make_array<T, const N: usize>() -> [T; N] {
    [T::default(); N]
}
```

### 关联常量

```rust
trait Consts {
    const VALUE: i32;
}
```

### Trait 对象

```rust
let obj: &dyn Trait = &impl_type;
```

### Blanket Impl

```rust
impl<T> MyTrait for T where T: OtherTrait {}
```

## 单元测试

详见 `tests/rust_features/26_latest_features_quick_ref.rs`
