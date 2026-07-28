//! 简历管理系统 — 核心库
//!
//! 统一管理所有模块的导出。各模块间的相互引用通过 crate 路径完成，
//! main.rs 通过 `use resume::xxx` 引用此库中的模块。

pub mod cli;
pub mod entry;
pub mod store;
pub mod template;
