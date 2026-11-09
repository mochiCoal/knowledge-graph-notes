use rusqlite::{Connection, Row};
use tauri::AppHandle;

use crate::db::{db_error, get_connection, table_counts};
use crate::models::dashboard::{DashboardOverview, RecentKnowledgePoint, RecentNote};
use crate::point_path::{load_point_index, path_titles};
use crate::text::{count_characters, excerpt};

/// 首页展示的最近编辑笔记数量
const RECENT_NOTE_LIMIT: i64 = 5;
/// 首页展示的最近创建知识点数量
const RECENT_POINT_LIMIT: i64 = 5;
/// 首页列表摘要的最大字符数
const RECENT_EXCERPT_MAX: usize = 96;

fn map_recent_note(row: &Row<'_>) -> rusqlite::Result<RecentNote> {
    let content: String = row.get(3)?;

    Ok(RecentNote {
        id: row.get(0)?,
        knowledge_point_id: row.get(1)?,
        title: row.get(2)?,
        excerpt: excerpt(&content, RECENT_EXCERPT_MAX),
        knowledge_point_title: row.get(4)?,
        course_id: row.get(5)?,
        course_name: row.get(6)?,
        character_count: count_characters(&content),
        updated_at: row.get(7)?,
    })
}

fn recent_notes(conn: &Connection, limit: i64) -> Result<Vec<RecentNote>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT n.id, n.knowledge_point_id, n.title, n.content,
                    kp.title, c.id, c.name, n.updated_at
             FROM notes n
             INNER JOIN knowledge_points kp ON kp.id = n.knowledge_point_id
             INNER JOIN courses c ON c.id = kp.course_id
             ORDER BY n.updated_at DESC, n.created_at DESC
             LIMIT ?1",
        )
        .map_err(|e| db_error("读取最近笔记", e))?;

    stmt.query_map([limit], map_recent_note)
        .map_err(|e| db_error("读取最近笔记", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取最近笔记", e))
}

fn recent_points(conn: &Connection, limit: i64) -> Result<Vec<RecentKnowledgePoint>, String> {
    let parents = load_point_index(conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT kp.id, kp.course_id, c.name, kp.title, kp.description,
                    kp.parent_id, kp.created_at,
                    (SELECT COUNT(*) FROM notes n WHERE n.knowledge_point_id = kp.id)
             FROM knowledge_points kp
             INNER JOIN courses c ON c.id = kp.course_id
             ORDER BY kp.created_at DESC, kp.id ASC
             LIMIT ?1",
        )
        .map_err(|e| db_error("读取最近知识点", e))?;

    let rows = stmt
        .query_map([limit], |row| {
            let description: String = row.get(4)?;
            let parent_id: Option<String> = row.get(5)?;

            Ok(RecentKnowledgePoint {
                id: row.get(0)?,
                course_id: row.get(1)?,
                course_name: row.get(2)?,
                title: row.get(3)?,
                excerpt: excerpt(&description, RECENT_EXCERPT_MAX),
                path: path_titles(&parents, parent_id.as_deref()),
                parent_id,
                note_count: row.get(7)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| db_error("读取最近知识点", e))?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取最近知识点", e))
}

/// 汇总首页需要的数据（统计数字 + 两组最近记录）
pub(crate) fn dashboard_overview(conn: &Connection) -> Result<DashboardOverview, String> {
    Ok(DashboardOverview {
        stats: table_counts(conn)?,
        recent_notes: recent_notes(conn, RECENT_NOTE_LIMIT)?,
        recent_points: recent_points(conn, RECENT_POINT_LIMIT)?,
    })
}

/// 首页概览：课程 / 知识点 / 笔记 / 标签 / 关联数量，最近编辑的笔记与最近创建的知识点
#[tauri::command]
pub fn get_dashboard_overview(app: AppHandle) -> Result<DashboardOverview, String> {
    let conn = get_connection(&app)?;
    dashboard_overview(&conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        let conn = Connection::open_in_memory().expect("打开内存数据库失败");
        conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
            .expect("初始化数据库失败");
        conn.execute_batch(include_str!("../../migrations/002_relations.sql"))
            .expect("执行关联迁移失败");
        conn
    }

    fn seed(conn: &Connection) {
        conn.execute_batch(
            "
            INSERT INTO courses (id, name, description, created_at, updated_at) VALUES
                ('c1', '数据结构', '线性结构与算法', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
                ('c2', '操作系统', '', '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');

            INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at) VALUES
                ('kp1', 'c1', NULL, '线性表', '顺序存储与链式存储', '2026-01-01T10:00:00Z', '2026-01-01T10:00:00Z'),
                ('kp2', 'c1', 'kp1', '链表', '**单链表** 与双向链表', '2026-01-02T10:00:00Z', '2026-01-02T10:00:00Z'),
                ('kp3', 'c2', NULL, '进程', '进程状态与调度', '2026-01-03T10:00:00Z', '2026-01-03T10:00:00Z');

            INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at) VALUES
                ('n1', 'kp2', '链表笔记', '# 链表\\n\\n插入与删除', '2026-01-01T00:00:00Z', '2026-01-05T00:00:00Z'),
                ('n2', 'kp3', '进程笔记', '进程有五种状态。', '2026-01-02T00:00:00Z', '2026-01-06T00:00:00Z');
            ",
        )
        .expect("写入测试数据失败");
    }

    #[test]
    fn overview_counts_all_tables_and_orders_recent_items() {
        let conn = test_connection();
        seed(&conn);

        let overview = dashboard_overview(&conn).expect("读取首页概览失败");
        assert_eq!(overview.stats.course_count, 2);
        assert_eq!(overview.stats.knowledge_point_count, 3);
        assert_eq!(overview.stats.note_count, 2);
        assert_eq!(overview.stats.tag_count, 0);
        assert_eq!(overview.stats.relation_count, 0);

        let note_ids: Vec<&str> = overview
            .recent_notes
            .iter()
            .map(|note| note.id.as_str())
            .collect();
        assert_eq!(note_ids, vec!["n2", "n1"]);

        let first = &overview.recent_notes[1];
        assert_eq!(first.knowledge_point_title, "链表");
        assert_eq!(first.course_name, "数据结构");
        assert!(!first.excerpt.contains('#'));
    }

    #[test]
    fn recent_points_expose_course_and_ancestor_path() {
        let conn = test_connection();
        seed(&conn);

        let overview = dashboard_overview(&conn).expect("读取首页概览失败");
        let point_ids: Vec<&str> = overview
            .recent_points
            .iter()
            .map(|point| point.id.as_str())
            .collect();
        assert_eq!(point_ids, vec!["kp3", "kp2", "kp1"]);

        let linked = &overview.recent_points[1];
        assert_eq!(linked.title, "链表");
        assert_eq!(linked.course_name, "数据结构");
        assert_eq!(linked.path, vec!["线性表"]);
        assert!(linked.excerpt.contains("单链表"));
        assert_eq!(linked.note_count, 1);

        assert!(overview.recent_points[0].path.is_empty());
    }

    #[test]
    fn empty_database_returns_zero_stats_and_empty_lists() {
        let conn = test_connection();

        let overview = dashboard_overview(&conn).expect("读取首页概览失败");
        assert_eq!(overview.stats.course_count, 0);
        assert_eq!(overview.stats.note_count, 0);
        assert!(overview.recent_notes.is_empty());
        assert!(overview.recent_points.is_empty());
    }
}
