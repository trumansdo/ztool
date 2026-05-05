//! # Rust 语法特性测试套件 — 入口文件
//!
//! 从入门到精通的学习路线（编号与 docs/notes/ 笔记一一对应）：
//! `入门基础 → 核心基础 → 函数式与元编程 → 并发与异步 → 系统编程 → 现代特性`
//!
//! ## Rust 概念 — `#[path]` 属性
//! `#[path = "rust_features/01_variables_types.rs"]` 允许自定义模块文件路径，
//! 绕过默认的文件系统查找规则。这里将 34 个编号测试文件按语义顺序重新组织。
//!
//! ## 学习路线图
//! | 阶段 | 编号 | 模块 |
//! |------|------|------|
//! | 入门基础 | 01-09 | variables_types / strings / ownership_borrowing / arrays_and_vecs / control_flow / structs_enums / pattern_matching / error_handling |
//! | 核心基础 | 10-15 | iterators / collections / type_system / generics / lifetimes / smart_pointers |
//! | 函数式与元编程 | 16-19 | functions_closures / macros / macro_fragments / tuple_iterator |
//! | 并发与异步 | 20-22 | async_basics / async_closures / concurrency |
//! | 系统编程 | 23-27 | unsafe_programming / ffi / testing / file_and_io / conditional_compilation |
//! | 现代特性 | 28-35 | let_chains / if_let_temporary_scope / diagnostic_attributes / precise_capturing / reserved_keywords / edition_2024_safety / latest_features / comprehensive |

#[path = "rust_features/04_arrays_and_vecs.rs"]
mod arrays_and_vecs;
#[path = "rust_features/20_async_basics.rs"]
mod async_basics;
#[path = "rust_features/21_async_closures.rs"]
mod async_closures;
#[path = "rust_features/11_collection_operations.rs"]
mod collection_operations;
#[path = "rust_features/35_comprehensive_tests.rs"]
mod comprehensive_tests;
#[path = "rust_features/22_concurrency.rs"]
mod concurrency;
#[path = "rust_features/27_conditional_compilation.rs"]
mod conditional_compilation;
#[path = "rust_features/06_control_flow.rs"]
mod control_flow;
#[path = "rust_features/30_diagnostic_attributes.rs"]
mod diagnostic_attributes;
#[path = "rust_features/33_edition_2024_safety.rs"]
mod edition_2024_safety;
#[path = "rust_features/09_error_handling_basics.rs"]
mod error_handling_basics;
#[path = "rust_features/24_ffi.rs"]
mod ffi;
#[path = "rust_features/26_file_and_io.rs"]
mod file_and_io;
#[path = "rust_features/16_functions_closures.rs"]
mod functions_closures;
#[path = "rust_features/13_generics.rs"]
mod generics;
#[path = "rust_features/29_if_let_temporary_scope.rs"]
mod if_let_temporary_scope;
#[path = "rust_features/10_iterators.rs"]
mod iterators;
#[path = "rust_features/34_latest_features_quick_ref.rs"]
mod latest_features_quick_ref;
#[path = "rust_features/28_let_chains.rs"]
mod let_chains;
#[path = "rust_features/14_lifetimes_and_rpit.rs"]
mod lifetimes_and_rpit;
#[path = "rust_features/18_macro_fragment_specifiers.rs"]
mod macro_fragment_specifiers;
#[path = "rust_features/17_macros.rs"]
mod macros;
#[path = "rust_features/03_ownership_borrowing.rs"]
mod ownership_borrowing;
#[path = "rust_features/08_pattern_matching.rs"]
mod pattern_matching;
#[path = "rust_features/31_precise_capturing.rs"]
mod precise_capturing;
#[path = "rust_features/32_reserved_keywords.rs"]
mod reserved_keywords;
#[path = "rust_features/15_smart_pointers.rs"]
mod smart_pointers;
#[path = "rust_features/02_strings.rs"]
mod strings;
#[path = "rust_features/07_structs_enums.rs"]
mod structs_enums;
#[path = "rust_features/25_testing.rs"]
mod testing;
#[path = "rust_features/19_tuple_iterator.rs"]
mod tuple_iterator;
#[path = "rust_features/12_type_system.rs"]
mod type_system;
#[path = "rust_features/23_unsafe_programming.rs"]
mod unsafe_programming;
#[path = "rust_features/01_variables_types.rs"]
mod variables_types;
