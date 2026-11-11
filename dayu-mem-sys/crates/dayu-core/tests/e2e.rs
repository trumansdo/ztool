//! 聚合级测试：跨 crate 串起来的完整用例
//!
//! 与 `tests/http_e2e.rs`（单模块：core 对外 HTTP 嵌入的端到端）分开：
//! 这里一个用例要串 core + flow + storage + judge，断言的是「整条链路对不对」，
//! 而不是「某个 crate 的契约对不对」。
//!
//! 子模块放在 `e2e/` 目录下，本入口只做声明。用 `#[path]` 显式指定路径——
//! 目录名与入口文件同名时，rustc 的默认模块查找会退化到 `tests/` 下。

#[path = "e2e/usecases.rs"]
mod usecases;
