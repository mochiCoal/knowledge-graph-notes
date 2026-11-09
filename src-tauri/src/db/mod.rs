use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Manager};

/// 数据库文件名（位于 Tauri `app_data_dir()` 下）
const DATABASE_FILE_NAME: &str = "knowledge_graph.db";

/// 获取 SQLite 数据库文件的绝对路径，并确保所在目录存在。
pub fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败：{}", e))?;

    fs::create_dir_all(&app_dir).map_err(|e| format!("创建数据库目录失败：{}", e))?;

    Ok(app_dir.join(DATABASE_FILE_NAME))
}

/// 打开指定位置的数据库连接（启用外键约束）。
///
/// `busy_timeout` 让备份 / 恢复等文件操作与正常读写短暂竞争时等待锁释放，
/// 而不是直接把「database is locked」抛给用户；外键约束保证异常数据无法绕过
/// 校验写入孤儿记录。
pub fn open_connection(db_path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("打开数据库失败：{}", e))?;

    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")
        .map_err(|e| format!("配置数据库连接失败：{}", e))?;

    Ok(conn)
}

/// 获取 SQLite 数据库连接
pub fn get_connection(app: &AppHandle) -> Result<Connection, String> {
    let db_path = database_path(app)?;
    open_connection(&db_path)
}

/// 数据库中各业务表的记录数
///
/// 首页统计与设置页的数据库信息共用同一份查询，避免两处统计口径不一致。
/// 使用 `serde(flatten)` 展开到前端对象中，字段名与 TypeScript 类型保持一致。
#[derive(Debug, Clone, Serialize)]
pub struct TableCounts {
    pub course_count: i64,
    pub knowledge_point_count: i64,
    pub note_count: i64,
    pub tag_count: i64,
    pub relation_count: i64,
}

/// 统计各业务表的数据量
pub fn table_counts(conn: &Connection) -> Result<TableCounts, String> {
    let count = |table: &str| -> Result<i64, String> {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .map_err(|e| db_error(&format!("统计数据表 {table}"), e))
    };

    Ok(TableCounts {
        course_count: count("courses")?,
        knowledge_point_count: count("knowledge_points")?,
        note_count: count("notes")?,
        tag_count: count("tags")?,
        relation_count: count("knowledge_relations")?,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("kgn-db-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&path).expect("创建临时目录失败");
            Self { path }
        }

        fn join(&self, name: &str) -> PathBuf {
            self.path.join(name)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn run_migrations(conn: &Connection) {
        for (name, sql) in MIGRATIONS {
            conn.execute_batch(sql)
                .unwrap_or_else(|e| panic!("执行迁移 {name} 失败：{e}"));
        }
    }

    fn seed_course_with_note(conn: &Connection) {
        conn.execute_batch(
            "
            INSERT INTO courses (id, name, description, created_at, updated_at)
            VALUES ('c1', '数据结构', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');

            INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at)
            VALUES ('kp1', 'c1', NULL, '链表', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');

            INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at)
            VALUES ('n1', 'kp1', '链表笔记', '插入 O(1)', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
            ",
        )
        .expect("写入测试数据失败");
    }

    #[test]
    fn file_database_persists_across_connections() {
        let dir = TempDir::new();
        let db_path = dir.join("knowledge_graph.db");

        {
            let conn = open_connection(&db_path).expect("打开数据库失败");
            run_migrations(&conn);
            seed_course_with_note(&conn);
            // 连接随作用域关闭，模拟应用退出
        }

        let conn = open_connection(&db_path).expect("重新打开数据库失败");
        let counts = table_counts(&conn).expect("统计数据失败");
        assert_eq!(counts.course_count, 1);
        assert_eq!(counts.knowledge_point_count, 1);
        assert_eq!(counts.note_count, 1);

        let title: String = conn
            .query_row("SELECT title FROM notes WHERE id = 'n1'", [], |row| {
                row.get(0)
            })
            .expect("读取笔记失败");
        assert_eq!(title, "链表笔记");
    }

    #[test]
    fn migrations_are_idempotent_and_keep_data() {
        let conn = Connection::open_in_memory().expect("打开内存数据库失败");
        run_migrations(&conn);
        seed_course_with_note(&conn);

        // 模拟每次启动都重放迁移：结构不报错、数据与内置字典都不重复
        run_migrations(&conn);

        let counts = table_counts(&conn).expect("统计数据失败");
        assert_eq!(counts.course_count, 1);
        assert_eq!(counts.note_count, 1);

        let types: i64 = conn
            .query_row("SELECT COUNT(*) FROM relation_types", [], |row| row.get(0))
            .expect("统计关联类型失败");
        assert_eq!(types, 5, "内置关联类型不应因重放迁移而重复");
    }

    #[test]
    fn foreign_keys_reject_orphan_rows() {
        let conn = Connection::open_in_memory().expect("打开内存数据库失败");
        run_migrations(&conn);

        // 指向不存在课程的知识点必须被外键拒绝，而不是写入孤儿数据
        let orphan = conn.execute(
            "INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at)
             VALUES ('kp-x', 'missing', NULL, '孤儿', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        );
        assert!(orphan.is_err(), "缺少外键约束时会写入孤儿知识点");
    }

    #[test]
    fn open_connection_enables_foreign_keys_and_busy_timeout() {
        let dir = TempDir::new();
        let db_path = dir.join("knowledge_graph.db");
        let conn = open_connection(&db_path).expect("打开数据库失败");

        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("读取外键设置失败");
        assert_eq!(foreign_keys, 1);

        let busy_timeout: i64 = conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .expect("读取忙等待设置失败");
        assert_eq!(busy_timeout, 5000);
    }

    #[test]
    fn table_counts_on_empty_database_are_zero() {
        let conn = Connection::open_in_memory().expect("打开内存数据库失败");
        run_migrations(&conn);

        let counts = table_counts(&conn).expect("统计数据失败");
        assert_eq!(counts.course_count, 0);
        assert_eq!(counts.knowledge_point_count, 0);
        assert_eq!(counts.note_count, 0);
        assert_eq!(counts.tag_count, 0);
        assert_eq!(counts.relation_count, 0);
    }

    #[test]
    fn escape_like_escapes_only_wildcards() {
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a_b\\c"), "a\\_b\\\\c");
        assert_eq!(escape_like("普通 关键字"), "普通 关键字");
        assert_eq!(escape_like(""), "");
    }
}
