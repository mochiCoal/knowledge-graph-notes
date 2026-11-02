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

/// 初始化数据库并执行建表 SQL
pub fn initialize_database(app: &AppHandle) -> Result<(), String> {
    let conn = get_connection(app)?;

    // 读取 src-tauri/migrations/001_initial.sql
    let migration_sql = include_str!("../../migrations/001_initial.sql");

    conn.execute_batch(migration_sql)
        .map_err(|e| format!("执行数据库初始化 SQL 失败：{}", e))?;

    println!("SQLite 数据库初始化成功！");

    Ok(())
}

/// 把底层数据库错误转换成可直接展示给用户的中文提示
pub fn db_error(context: &str, error: impl std::fmt::Display) -> String {
    format!("{context}失败：{error}")
}
