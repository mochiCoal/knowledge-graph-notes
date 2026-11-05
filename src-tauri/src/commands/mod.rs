//! # 命令层（Tauri Commands）
//!
//! 集中存放暴露给前端的 `#[tauri::command]` 函数，按业务领域拆分子模块，
//! 由 `lib.rs` 统一注册到 `invoke_handler`。
//!
//! 约定：
//! 1. 命令函数只负责参数接收、调用数据库访问逻辑、组装返回值，不直接拼接复杂 SQL；
//! 2. 统一返回 `Result<T, String>`，错误信息使用中文，前端可直接展示；
//! 3. 新增命令后必须在 `lib.rs` 的 `invoke_handler` 中注册，否则前端调用会报错。
//!
//! 当前已实现：课程、知识点的完整增删改查（含知识点层级查询与课程迁移），
//! 以及笔记、标签、知识点关联的基础命令。

pub mod course;
pub mod knowledge_point;
pub mod note;
pub mod relation;
pub mod tag;
