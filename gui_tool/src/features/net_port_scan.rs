//! 端口扫描模块
//!
//! 基于 rayon 并行库的高性能 TCP 端口扫描器。
//!
//! # 技术要点
//!
//! ## rayon —— 数据并行库
//! `rayon::prelude::*` 提供了 `.par_iter()` 方法，
//! 自动将迭代器操作分配到多个 CPU 核心并行执行。
//! 无需手动管理线程 —— rayon 使用 work-stealing 全局线程池。
//!
//! ## 并发与线程安全
//! - `Mutex<T>`: 互斥锁，保证多个线程安全地访问共享数据
//! - `Send + Sync`: Rust 的线程安全标记 trait
//!   - `Send`: 类型可以安全地将所有权转移到其他线程
//!   - `Sync`: 类型的共享引用可以安全地在多线程间使用

pub mod scan;
pub mod update;
pub mod view;

pub use update::{update, Msg, NetScanner, ScanMode};
pub use view::view;
