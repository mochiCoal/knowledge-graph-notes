use rusqlite::Connection;
use std::fs;
use tauri::{AppHandle, Manager};

/// 获取 SQLite 数据库连接
pub fn get_connection(app: &AppHandle) -> Result<Connection, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败：{}", e))?;

    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("创建数据库目录失败：{}", e))?;

    let db_path = app_dir.join("knowledge_graph.db");

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("打开数据库失败：{}", e))?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| format!("启用外键约束失败：{}", e))?;

    Ok(conn)
}

/// 按顺序执行的数据库迁移。
///
/// 迁移脚本全部使用 `CREATE TABLE IF NOT EXISTS` / `INSERT OR IGNORE` 等
/// 幂等语句，因此每次启动都可以安全重放；新增结构变化时在这里追加文件，
/// 不修改已经投入使用的迁移。
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_initial",
        include_str!("../../migrations/001_initial.sql"),
    ),
    (
        "002_relations",
        include_str!("../../migrations/002_relations.sql"),
    ),
];

/// 初始化数据库并按顺序执行迁移
pub fn initialize_database(app: &AppHandle) -> Result<(), String> {
    let conn = get_connection(app)?;

    for (name, sql) in MIGRATIONS {
        conn.execute_batch(sql)
            .map_err(|e| format!("执行数据库迁移 {name} 失败：{e}"))?;
    }

    println!("SQLite 数据库初始化成功！");

    Ok(())
}

/// 把底层数据库错误转换成可直接展示给用户的中文提示
pub fn db_error(context: &str, error: impl std::fmt::Display) -> String {
    format!("{context}失败：{error}")
}

/// 转义 LIKE 通配符，让用户输入的关键字按字面量匹配
pub fn escape_like(keyword: &str) -> String {
    let mut escaped = String::with_capacity(keyword.len());

    for ch in keyword.chars() {
        if matches!(ch, '%' | '_' | '\\') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }

    escaped
}
