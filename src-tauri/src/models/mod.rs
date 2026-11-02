//! # 数据模型层
//!
//! 存放与 SQLite 表结构一一对应的 Rust 结构体，以及命令层使用的请求 / 响应结构。
//!
//! 约定：
//! 1. 字段使用 snake_case，与数据库列名保持一致，减少映射代码；
//! 2. 查询结果结构体实现 `serde::Serialize`，写入参数实现 `serde::Deserialize`；
//! 3. 时间字段以 ISO 8601 字符串存储，与 `chrono` 序列化格式保持一致。
//!
//! 当前已实现：课程、知识点、笔记、标签、知识点关联。

pub mod course;
pub mod knowledge_point;
pub mod note;
pub mod relation;
pub mod tag;
