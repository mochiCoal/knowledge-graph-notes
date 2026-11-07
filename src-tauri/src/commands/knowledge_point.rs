use std::collections::{HashMap, HashSet};

use rusqlite::{params, Connection, OptionalExtension, Row};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{db_error, get_connection};
use crate::models::knowledge_point::{
    KnowledgePoint, KnowledgePointDeletion, KnowledgePointDetail, KnowledgePointNode,
    KnowledgePointPathItem,
};

/// 知识点查询统一使用的 SELECT 片段，字段顺序与 [`map_knowledge_point`] 一致。
const POINT_SELECT: &str = "
SELECT id, course_id, parent_id, title, description, created_at, updated_at
FROM knowledge_points
";

fn map_knowledge_point(row: &Row<'_>) -> rusqlite::Result<KnowledgePoint> {
    Ok(KnowledgePoint {
        id: row.get(0)?,
        course_id: row.get(1)?,
        parent_id: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

/// 查询全部知识点（按创建时间升序，前端可按 course_id / parent_id 自行组装树）
fn list_all(conn: &Connection) -> Result<Vec<KnowledgePoint>, String> {
    let sql = format!("{POINT_SELECT} ORDER BY created_at ASC");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("读取知识点列表", e))?;

    stmt.query_map([], map_knowledge_point)
        .map_err(|e| db_error("读取知识点列表", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取知识点列表", e))
}

/// 查询某门课程下的全部知识点（按创建时间升序）
fn list_by_course(conn: &Connection, course_id: &str) -> Result<Vec<KnowledgePoint>, String> {
    let sql = format!("{POINT_SELECT} WHERE course_id = ?1 ORDER BY created_at ASC");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("读取知识点列表", e))?;

    stmt.query_map([course_id], map_knowledge_point)
        .map_err(|e| db_error("读取知识点列表", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取知识点列表", e))
}

/// 读取单个知识点；不存在时返回 `None`
fn find_point(conn: &Connection, point_id: &str) -> Result<Option<KnowledgePoint>, String> {
    let sql = format!("{POINT_SELECT} WHERE id = ?1");
    conn.query_row(&sql, [point_id], map_knowledge_point)
        .optional()
        .map_err(|e| db_error("读取知识点", e))
}

/// 读取单个知识点；不存在时返回可直接展示的中文错误
fn require_point(conn: &Connection, point_id: &str) -> Result<KnowledgePoint, String> {
    find_point(conn, point_id)?.ok_or_else(|| "知识点不存在或已被删除，请刷新后重试".to_string())
}

/// 读取课程名称，同时用作「课程是否存在」的校验
fn course_name(conn: &Connection, course_id: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT name FROM courses WHERE id = ?1",
        [course_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| db_error("读取课程", e))?
    .ok_or_else(|| "课程不存在，请刷新后重试".to_string())
}

/// 统计直接挂在某个知识点上的笔记数量
fn count_notes(conn: &Connection, point_id: &str) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE knowledge_point_id = ?1",
        [point_id],
        |row| row.get(0),
    )
    .map_err(|e| db_error("统计知识点笔记", e))
}

/// 统计知识点的语义关联数量（出边、入边）
fn count_relations(conn: &Connection, point_id: &str) -> Result<(i64, i64), String> {
    let outgoing = conn
        .query_row(
            "SELECT COUNT(*) FROM knowledge_relations WHERE source_id = ?1",
            [point_id],
            |row| row.get(0),
        )
        .map_err(|e| db_error("统计知识点关联", e))?;

    let incoming = conn
        .query_row(
            "SELECT COUNT(*) FROM knowledge_relations WHERE target_id = ?1",
            [point_id],
            |row| row.get(0),
        )
        .map_err(|e| db_error("统计知识点关联", e))?;

    Ok((outgoing, incoming))
}

/// 校验并规范化知识点输入（名称必填且不超过 80 字，描述不超过 1000 字）
fn validate_point_input(title: &str, description: &str) -> Result<(String, String), String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("知识点名称不能为空".into());
    }
    if title.chars().count() > 80 {
        return Err("知识点名称不能超过 80 个字符".into());
    }

    let description = description.trim();
    if description.chars().count() > 1000 {
        return Err("知识点描述不能超过 1000 个字符".into());
    }

    Ok((title.to_string(), description.to_string()))
}

/// 递归收集某个知识点及其全部后代的 ID（父节点在子节点之前，成环时自动停止）
fn collect_subtree_ids(conn: &Connection, root_id: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT id, parent_id FROM knowledge_points")
        .map_err(|e| db_error("读取知识点层级", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .map_err(|e| db_error("读取知识点层级", e))?;

    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let (id, parent_id) = row.map_err(|e| db_error("读取知识点层级", e))?;
        if let Some(parent_id) = parent_id {
            children.entry(parent_id).or_default().push(id);
        }
    }

    let mut ordered = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut pending = vec![root_id.to_string()];

    while let Some(id) = pending.pop() {
        if !visited.insert(id.clone()) {
            continue;
        }
        ordered.push(id.clone());

        if let Some(child_ids) = children.get(&id) {
            pending.extend(child_ids.iter().cloned());
        }
    }

    Ok(ordered)
}

/// 校验父知识点：必须存在、属于同一课程，且不能形成环
///
/// `moving_id` 为正在移动 / 修改的知识点，用于拒绝「自己作为父节点」和
/// 「移动到自己的子知识点下」两种非法层级。
fn ensure_parent_available(
    conn: &Connection,
    course_id: &str,
    parent_id: Option<&str>,
    moving_id: Option<&str>,
) -> Result<(), String> {
    let Some(parent_id) = parent_id else {
        return Ok(());
    };

    if Some(parent_id) == moving_id {
        return Err("不能把知识点设为自己的父知识点".into());
    }

    let parent = require_point(conn, parent_id)?;
    if parent.course_id != course_id {
        return Err("父知识点不属于该课程，请重新选择".into());
    }

    if let Some(moving_id) = moving_id {
        let subtree = collect_subtree_ids(conn, moving_id)?;
        if subtree.iter().any(|id| id == parent_id) {
            return Err("不能把知识点移动到它自己的子知识点下".into());
        }
    }

    Ok(())
}

/// 把扁平的知识点列表组装成层级树
///
/// 父节点缺失（已被删除或属于其它课程）的知识点会作为顶层节点展示；
/// 数据异常形成环时，环上的节点只会出现一次，不会丢失。
fn build_tree(
    points: &[KnowledgePoint],
    note_counts: &HashMap<String, i64>,
) -> Vec<KnowledgePointNode> {
    let ids: HashSet<&str> = points.iter().map(|point| point.id.as_str()).collect();
    let mut children: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut roots: Vec<usize> = Vec::new();

    for (index, point) in points.iter().enumerate() {
        match point.parent_id.as_deref() {
            Some(parent_id) if ids.contains(parent_id) => {
                children.entry(parent_id).or_default().push(index);
            }
            _ => roots.push(index),
        }
    }

    fn build_node<'a>(
        points: &'a [KnowledgePoint],
        children: &HashMap<&'a str, Vec<usize>>,
        note_counts: &HashMap<String, i64>,
        index: usize,
        depth: i64,
        visited: &mut HashSet<String>,
    ) -> KnowledgePointNode {
        let point = &points[index];
        visited.insert(point.id.clone());

        let child_indices: Vec<usize> = match children.get(point.id.as_str()) {
            Some(indices) => indices
                .iter()
                .copied()
                .filter(|child| !visited.contains(&points[*child].id))
                .collect(),
            None => Vec::new(),
        };

        let child_nodes: Vec<KnowledgePointNode> = child_indices
            .into_iter()
            .map(|child| build_node(points, children, note_counts, child, depth + 1, visited))
            .collect();

        KnowledgePointNode {
            point: point.clone(),
            depth,
            child_count: child_nodes.len() as i64,
            note_count: note_counts.get(&point.id).copied().unwrap_or(0),
            children: child_nodes,
        }
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut nodes = Vec::new();

    for index in roots {
        if visited.contains(&points[index].id) {
            continue;
        }
        nodes.push(build_node(
            points,
            &children,
            note_counts,
            index,
            0,
            &mut visited,
        ));
    }

    // 兜底：父节点成环等异常数据不能被静默丢弃，按顶层节点补齐
    for index in 0..points.len() {
        if visited.contains(&points[index].id) {
            continue;
        }
        nodes.push(build_node(
            points,
            &children,
            note_counts,
            index,
            0,
            &mut visited,
        ));
    }

    nodes
}

/// 读取某门课程的知识点树（含层级深度、子节点与笔记数量）
fn load_tree(conn: &Connection, course_id: &str) -> Result<Vec<KnowledgePointNode>, String> {
    let course_id = course_id.trim();
    course_name(conn, course_id)?;

    let points = list_by_course(conn, course_id)?;

    let mut note_counts: HashMap<String, i64> = HashMap::new();
    let mut stmt = conn
        .prepare(
            "SELECT kp.id,
                    (SELECT COUNT(*) FROM notes n WHERE n.knowledge_point_id = kp.id)
             FROM knowledge_points kp
             WHERE kp.course_id = ?1",
        )
        .map_err(|e| db_error("统计知识点笔记", e))?;

    let rows = stmt
        .query_map([course_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| db_error("统计知识点笔记", e))?;

    for row in rows {
        let (id, count) = row.map_err(|e| db_error("统计知识点笔记", e))?;
        note_counts.insert(id, count);
    }

    Ok(build_tree(&points, &note_counts))
}

/// 读取知识点详情：面包屑路径、层级、子节点与笔记统计
fn load_detail(conn: &Connection, point_id: &str) -> Result<KnowledgePointDetail, String> {
    let point = require_point(conn, point_id)?;
    let course = course_name(conn, &point.course_id)?;

    // 从当前节点向上回溯，遇到环或已删除的父节点时停止
    let mut path: Vec<KnowledgePointPathItem> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut current: Option<KnowledgePoint> = Some(point.clone());

    while let Some(item) = current {
        if !visited.insert(item.id.clone()) {
            break;
        }

        path.push(KnowledgePointPathItem {
            id: item.id.clone(),
            title: item.title.clone(),
        });

        current = match item.parent_id.as_deref() {
            Some(parent_id) => find_point(conn, parent_id)?,
            None => None,
        };
    }
    path.reverse();

    let subtree = collect_subtree_ids(conn, point_id)?;
    let mut subtree_note_count = 0i64;
    for id in &subtree {
        subtree_note_count += count_notes(conn, id)?;
    }

    let child_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM knowledge_points WHERE parent_id = ?1",
            [point_id],
            |row| row.get(0),
        )
        .map_err(|e| db_error("统计子知识点", e))?;

    let (outgoing_relation_count, incoming_relation_count) = count_relations(conn, point_id)?;

    Ok(KnowledgePointDetail {
        point,
        course_name: course,
        depth: path.len().saturating_sub(1) as i64,
        path,
        child_count,
        descendant_count: subtree.len().saturating_sub(1) as i64,
        note_count: count_notes(conn, point_id)?,
        subtree_note_count,
        relation_count: outgoing_relation_count + incoming_relation_count,
        outgoing_relation_count,
        incoming_relation_count,
    })
}

/// 新增知识点（可指定父知识点形成层级）
fn create_point(
    conn: &Connection,
    course_id: &str,
    parent_id: Option<&str>,
    title: &str,
    description: &str,
) -> Result<KnowledgePoint, String> {
    let (title, description) = validate_point_input(title, description)?;

    let course_id = course_id.trim();
    course_name(conn, course_id)?;
    ensure_parent_available(conn, course_id, parent_id, None)?;

    let now = chrono::Utc::now().to_rfc3339();
    let point = KnowledgePoint {
        id: Uuid::new_v4().to_string(),
        course_id: course_id.to_string(),
        parent_id: parent_id.map(str::to_string),
        title,
        description,
        created_at: now.clone(),
        updated_at: now,
    };

    conn.execute(
        "INSERT INTO knowledge_points
            (id, course_id, parent_id, title, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            point.id,
            point.course_id,
            point.parent_id,
            point.title,
            point.description,
            point.created_at,
            point.updated_at
        ],
    )
    .map_err(|e| db_error("创建知识点", e))?;

    Ok(point)
}

/// 修改知识点（同时可调整父节点，即层级调整）
fn update_point(
    conn: &Connection,
    point_id: &str,
    parent_id: Option<&str>,
    title: &str,
    description: &str,
) -> Result<KnowledgePoint, String> {
    let existing = require_point(conn, point_id)?;
    let (title, description) = validate_point_input(title, description)?;
    ensure_parent_available(conn, &existing.course_id, parent_id, Some(point_id))?;

    let updated_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE knowledge_points
            SET title = ?1, description = ?2, parent_id = ?3, updated_at = ?4
          WHERE id = ?5",
        params![title, description, parent_id, updated_at, point_id],
    )
    .map_err(|e| db_error("修改知识点", e))?;

    require_point(conn, point_id)
}

/// 只调整层级（不改动名称与描述），供「降为子级 / 提升一级」使用
fn move_point(
    conn: &Connection,
    point_id: &str,
    parent_id: Option<&str>,
) -> Result<KnowledgePoint, String> {
    let existing = require_point(conn, point_id)?;
    if existing.parent_id.as_deref() == parent_id {
        return Ok(existing);
    }

    ensure_parent_available(conn, &existing.course_id, parent_id, Some(point_id))?;

    let updated_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE knowledge_points SET parent_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![parent_id, updated_at, point_id],
    )
    .map_err(|e| db_error("调整知识点层级", e))?;

    require_point(conn, point_id)
}

/// 迁移知识点所属课程（整棵子树跟随迁移，根节点成为新课程的顶层知识点）
fn move_point_to_course(
    conn: &Connection,
    point_id: &str,
    course_id: &str,
) -> Result<KnowledgePoint, String> {
    let existing = require_point(conn, point_id)?;
    let course_id = course_id.trim();
    course_name(conn, course_id)?;

    if existing.course_id == course_id {
        return Ok(existing);
    }

    let subtree = collect_subtree_ids(conn, point_id)?;
    let updated_at = chrono::Utc::now().to_rfc3339();

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| db_error("迁移知识点所属课程", e))?;

    {
        let mut stmt = tx
            .prepare("UPDATE knowledge_points SET course_id = ?1, updated_at = ?2 WHERE id = ?3")
            .map_err(|e| db_error("迁移知识点所属课程", e))?;

        for id in &subtree {
            stmt.execute(params![course_id, updated_at, id])
                .map_err(|e| db_error("迁移知识点所属课程", e))?;
        }
    }

    // 原父节点属于旧课程，迁移后必须断开，否则会形成跨课程的层级
    tx.execute(
        "UPDATE knowledge_points SET parent_id = NULL WHERE id = ?1",
        [point_id],
    )
    .map_err(|e| db_error("迁移知识点所属课程", e))?;

    tx.commit()
        .map_err(|e| db_error("迁移知识点所属课程", e))?;

    require_point(conn, point_id)
}

/// 删除知识点及其全部后代，并统计被级联删除的笔记数量
fn delete_point(conn: &Connection, point_id: &str) -> Result<KnowledgePointDeletion, String> {
    require_point(conn, point_id)?;
    let subtree = collect_subtree_ids(conn, point_id)?;

    let mut deleted_notes = 0i64;
    for id in &subtree {
        deleted_notes += count_notes(conn, id)?;
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| db_error("删除知识点", e))?;

    {
        let mut stmt = tx
            .prepare("DELETE FROM knowledge_points WHERE id = ?1")
            .map_err(|e| db_error("删除知识点", e))?;

        for id in &subtree {
            stmt.execute(params![id])
                .map_err(|e| db_error("删除知识点", e))?;
        }
    }

    tx.commit().map_err(|e| db_error("删除知识点", e))?;

    Ok(KnowledgePointDeletion {
        deleted_points: subtree.len() as i64,
        deleted_notes,
    })
}

/// 查询全部知识点（笔记工作区按课程 / 父节点自行组装目录）
#[tauri::command]
pub fn list_knowledge_points(app: AppHandle) -> Result<Vec<KnowledgePoint>, String> {
    let conn = get_connection(&app)?;
    list_all(&conn)
}

/// 查询某门课程下的全部知识点（知识点管理页的课程与知识点关联查询）
#[tauri::command]
pub fn list_knowledge_points_by_course(
    app: AppHandle,
    course_id: String,
) -> Result<Vec<KnowledgePoint>, String> {
    let conn = get_connection(&app)?;
    list_by_course(&conn, course_id.trim())
}

/// 查询知识点树（含层级深度、子节点与笔记数量）
#[tauri::command]
pub fn get_knowledge_point_tree(
    app: AppHandle,
    course_id: String,
) -> Result<Vec<KnowledgePointNode>, String> {
    let conn = get_connection(&app)?;
    load_tree(&conn, &course_id)
}

/// 查询知识点详情（面包屑路径、层级、子节点与笔记统计）
#[tauri::command]
pub fn get_knowledge_point_detail(
    app: AppHandle,
    knowledge_point_id: String,
) -> Result<KnowledgePointDetail, String> {
    let conn = get_connection(&app)?;
    load_detail(&conn, &knowledge_point_id)
}

/// 新增知识点，可指定父知识点形成层级
#[tauri::command]
pub fn create_knowledge_point(
    app: AppHandle,
    course_id: String,
    parent_id: Option<String>,
    title: String,
    description: String,
) -> Result<KnowledgePoint, String> {
    let conn = get_connection(&app)?;
    create_point(
        &conn,
        &course_id,
        parent_id.as_deref(),
        &title,
        &description,
    )
}

/// 修改知识点（名称、描述与父节点）
#[tauri::command]
pub fn update_knowledge_point(
    app: AppHandle,
    knowledge_point_id: String,
    parent_id: Option<String>,
    title: String,
    description: String,
) -> Result<KnowledgePoint, String> {
    let conn = get_connection(&app)?;
    update_point(
        &conn,
        &knowledge_point_id,
        parent_id.as_deref(),
        &title,
        &description,
    )
}

/// 调整知识点层级（父节点为空表示提升为顶层知识点）
#[tauri::command]
pub fn move_knowledge_point(
    app: AppHandle,
    knowledge_point_id: String,
    parent_id: Option<String>,
) -> Result<KnowledgePoint, String> {
    let conn = get_connection(&app)?;
    move_point(&conn, &knowledge_point_id, parent_id.as_deref())
}

/// 把知识点迁移到另一门课程（整棵子树跟随迁移）
#[tauri::command]
pub fn move_knowledge_point_to_course(
    app: AppHandle,
    knowledge_point_id: String,
    course_id: String,
) -> Result<KnowledgePoint, String> {
    let conn = get_connection(&app)?;
    move_point_to_course(&conn, &knowledge_point_id, &course_id)
}

/// 删除知识点及其全部后代
#[tauri::command]
pub fn delete_knowledge_point(
    app: AppHandle,
    knowledge_point_id: String,
) -> Result<KnowledgePointDeletion, String> {
    let conn = get_connection(&app)?;
    delete_point(&conn, &knowledge_point_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        let conn = Connection::open_in_memory().expect("打开内存数据库失败");
        conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
            .expect("初始化数据库失败");
        conn
    }

    fn insert_course(conn: &Connection, id: &str, name: &str) {
        conn.execute(
            "INSERT INTO courses (id, name, description, created_at, updated_at)
             VALUES (?1, ?2, '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            params![id, name],
        )
        .expect("插入课程失败");
    }

    fn insert_point(conn: &Connection, id: &str, course_id: &str, parent_id: Option<&str>, title: &str) {
        conn.execute(
            "INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            params![id, course_id, parent_id, title],
        )
        .expect("插入知识点失败");
    }

    fn insert_note(conn: &Connection, id: &str, point_id: &str) {
        conn.execute(
            "INSERT INTO notes
                (id, knowledge_point_id, title, content, created_at, updated_at)
             VALUES (?1, ?2, '笔记', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            params![id, point_id],
        )
        .expect("插入笔记失败");
    }

    fn flatten(nodes: &[KnowledgePointNode]) -> Vec<&KnowledgePointNode> {
        let mut result = Vec::new();
        for node in nodes {
            result.push(node);
            result.extend(flatten(&node.children));
        }
        result
    }

    fn seed_tree(conn: &Connection) {
        insert_course(conn, "c1", "数据结构");
        insert_course(conn, "c2", "操作系统");

        insert_point(conn, "k1", "c1", None, "线性表");
        insert_point(conn, "k2", "c1", Some("k1"), "链表");
        insert_point(conn, "k3", "c1", Some("k2"), "双向链表");
        insert_point(conn, "k4", "c1", None, "树");
        insert_point(conn, "k5", "c2", None, "进程");

        insert_note(conn, "n1", "k3");
        insert_note(conn, "n2", "k3");
        insert_note(conn, "n3", "k1");
    }

    #[test]
    fn tree_is_built_with_depth_children_and_note_counts() {
        let conn = test_connection();
        seed_tree(&conn);

        let tree = load_tree(&conn, "c1").expect("读取知识点树失败");
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].point.title, "线性表");
        assert_eq!(tree[0].depth, 0);
        assert_eq!(tree[0].child_count, 1);
        assert_eq!(tree[0].note_count, 1);

        let link_list = &tree[0].children[0];
        assert_eq!(link_list.point.title, "链表");
        assert_eq!(link_list.depth, 1);
        assert_eq!(link_list.children.len(), 1);

        let doubly = &link_list.children[0];
        assert_eq!(doubly.point.title, "双向链表");
        assert_eq!(doubly.depth, 2);
        assert_eq!(doubly.note_count, 2);

        assert_eq!(tree[1].point.title, "树");
        assert_eq!(tree[1].child_count, 0);

        assert!(load_tree(&conn, "missing").is_err());
    }

    #[test]
    fn tree_keeps_points_with_cyclic_parent_data() {
        let conn = test_connection();
        insert_course(&conn, "c1", "循环数据");
        insert_point(&conn, "a", "c1", None, "A");
        insert_point(&conn, "b", "c1", Some("a"), "B");

        // 制造 a -> b -> a 的环
        conn.execute("UPDATE knowledge_points SET parent_id = 'b' WHERE id = 'a'", [])
            .expect("构造环失败");

        let tree = load_tree(&conn, "c1").expect("读取知识点树失败");
        let nodes = flatten(&tree);
        assert_eq!(nodes.len(), 2, "环上的知识点不能被丢弃");
    }

    #[test]
    fn orphan_parent_is_shown_as_root_node() {
        let conn = test_connection();
        insert_course(&conn, "c1", "数据结构");
        insert_course(&conn, "c2", "操作系统");
        insert_point(&conn, "k1", "c1", None, "线性表");
        insert_point(&conn, "k5", "c2", None, "进程");
        // 父节点属于另一门课程：知识点树里应作为顶层节点兜底展示
        insert_point(&conn, "k2", "c1", Some("k5"), "异常知识点");

        let tree = load_tree(&conn, "c1").expect("读取知识点树失败");
        assert_eq!(tree.len(), 2);
        assert!(tree.iter().any(|node| node.point.id == "k2" && node.depth == 0));
    }

    #[test]
    fn detail_reports_path_depth_and_note_counts() {
        let conn = test_connection();
        seed_tree(&conn);

        let detail = load_detail(&conn, "k3").expect("读取知识点详情失败");
        assert_eq!(detail.point.title, "双向链表");
        assert_eq!(detail.course_name, "数据结构");
        assert_eq!(detail.depth, 2);
        assert_eq!(
            detail.path.iter().map(|item| item.title.as_str()).collect::<Vec<_>>(),
            vec!["线性表", "链表", "双向链表"]
        );
        assert_eq!(detail.child_count, 0);
        assert_eq!(detail.descendant_count, 0);
        assert_eq!(detail.note_count, 2);
        assert_eq!(detail.subtree_note_count, 2);

        let root = load_detail(&conn, "k1").expect("读取知识点详情失败");
        assert_eq!(root.child_count, 1);
        assert_eq!(root.descendant_count, 2);
        assert_eq!(root.note_count, 1);
        assert_eq!(root.subtree_note_count, 3);
    }

    #[test]
    fn detail_reports_relation_counts() {
        let conn = test_connection();
        seed_tree(&conn);

        conn.execute(
            "INSERT INTO knowledge_relations
                (id, source_id, target_id, relation_type, description, created_at)
             VALUES ('r1', 'k1', 'k2', 'related', '', '2026-01-01T00:00:00Z'),
                    ('r2', 'k3', 'k1', 'prerequisite', '', '2026-01-01T00:00:00Z')",
            [],
        )
        .expect("插入关联失败");

        let detail = load_detail(&conn, "k1").expect("读取知识点详情失败");
        assert_eq!(detail.relation_count, 2);
        assert_eq!(detail.outgoing_relation_count, 1);
        assert_eq!(detail.incoming_relation_count, 1);

        let none = load_detail(&conn, "k4").expect("读取知识点详情失败");
        assert_eq!(none.relation_count, 0);
    }

    #[test]
    fn point_input_is_trimmed_and_validated() {
        assert!(validate_point_input("   ", "").is_err());
        assert!(validate_point_input(&"点".repeat(81), "").is_err());
        assert!(validate_point_input("知识点", &"描".repeat(1001)).is_err());

        let (title, description) =
            validate_point_input("  链表  ", "  基础结构  ").expect("校验应通过");
        assert_eq!(title, "链表");
        assert_eq!(description, "基础结构");
    }

    #[test]
    fn tree_node_serializes_to_flat_json() {
        let conn = test_connection();
        seed_tree(&conn);

        let tree = load_tree(&conn, "c1").expect("读取知识点树失败");
        let json = serde_json::to_value(&tree).expect("序列化知识点树失败");
        let root = &json[0];

        // serde flatten：KnowledgePoint 的字段应与层级字段处于同一层
        assert_eq!(root["title"], "线性表");
        assert_eq!(root["course_id"], "c1");
        assert!(root["parent_id"].is_null());
        assert_eq!(root["depth"], 0);
        assert_eq!(root["note_count"], 1);
        assert_eq!(root["child_count"], 1);
        assert_eq!(root["children"][0]["title"], "链表");
        assert_eq!(root["children"][0]["children"][0]["title"], "双向链表");
    }

    #[test]
    fn detail_serializes_with_nested_point_and_path() {
        let conn = test_connection();
        seed_tree(&conn);

        let detail = load_detail(&conn, "k3").expect("读取知识点详情失败");
        let json = serde_json::to_value(&detail).expect("序列化知识点详情失败");

        assert_eq!(json["point"]["title"], "双向链表");
        assert_eq!(json["course_name"], "数据结构");
        assert_eq!(json["depth"], 2);
        assert_eq!(json["path"][0]["title"], "线性表");
        assert_eq!(json["path"][2]["title"], "双向链表");
        assert_eq!(json["subtree_note_count"], 2);
    }

    #[test]
    fn create_requires_existing_course_and_matching_parent() {
        let conn = test_connection();
        seed_tree(&conn);

        let created = create_point(&conn, "c1", Some("k1"), "  栈  ", "  后进先出  ")
            .expect("创建知识点失败");
        assert_eq!(created.title, "栈");
        assert_eq!(created.description, "后进先出");
        assert_eq!(created.parent_id.as_deref(), Some("k1"));

        assert!(create_point(&conn, "missing", None, "栈", "").is_err());
        // 父知识点属于另一门课程
        assert!(create_point(&conn, "c1", Some("k5"), "栈", "").is_err());
        // 父知识点不存在
        assert!(create_point(&conn, "c1", Some("nope"), "栈", "").is_err());
    }

    #[test]
    fn moving_point_rejects_self_and_descendant() {
        let conn = test_connection();
        seed_tree(&conn);

        assert!(move_point(&conn, "k1", Some("k1")).is_err());
        assert!(move_point(&conn, "k1", Some("k3")).is_err());
        assert!(move_point(&conn, "k1", Some("k2")).is_err());
        assert!(move_point(&conn, "k1", Some("k5")).is_err());

        let moved = move_point(&conn, "k3", Some("k1")).expect("调整层级失败");
        assert_eq!(moved.parent_id.as_deref(), Some("k1"));

        let promoted = move_point(&conn, "k3", None).expect("提升为顶层失败");
        assert_eq!(promoted.parent_id, None);
    }

    #[test]
    fn deleting_point_removes_subtree_and_notes() {
        let conn = test_connection();
        seed_tree(&conn);

        let deletion = delete_point(&conn, "k2").expect("删除知识点失败");
        assert_eq!(deletion.deleted_points, 2);
        assert_eq!(deletion.deleted_notes, 2);

        let remaining = list_by_course(&conn, "c1").expect("读取知识点列表失败");
        assert_eq!(
            remaining.iter().map(|point| point.id.as_str()).collect::<Vec<_>>(),
            vec!["k1", "k4"]
        );

        let notes: i64 = conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .expect("统计笔记失败");
        assert_eq!(notes, 1, "只剩 k1 上的笔记");
    }

    #[test]
    fn moving_to_another_course_moves_whole_subtree() {
        let conn = test_connection();
        seed_tree(&conn);

        let moved = move_point_to_course(&conn, "k2", "c2").expect("迁移课程失败");
        assert_eq!(moved.course_id, "c2");
        assert_eq!(moved.parent_id, None, "迁移后脱离原父节点");

        let target = list_by_course(&conn, "c2").expect("读取知识点列表失败");
        let ids: Vec<&str> = target.iter().map(|point| point.id.as_str()).collect();
        assert!(ids.contains(&"k5"));
        assert!(ids.contains(&"k2"));
        assert!(ids.contains(&"k3"));

        let source = list_by_course(&conn, "c1").expect("读取知识点列表失败");
        let source_ids: Vec<&str> = source.iter().map(|point| point.id.as_str()).collect();
        assert_eq!(source_ids, vec!["k1", "k4"]);

        // 课程不存在时应拒绝迁移
        assert!(move_point_to_course(&conn, "k2", "missing").is_err());
    }

    #[test]
    fn update_point_can_move_hierarchy_and_rename() {
        let conn = test_connection();
        seed_tree(&conn);

        let updated = update_point(&conn, "k3", Some("k4"), "  双链表  ", "  指向前后节点  ")
            .expect("修改知识点失败");
        assert_eq!(updated.title, "双链表");
        assert_eq!(updated.description, "指向前后节点");
        assert_eq!(updated.parent_id.as_deref(), Some("k4"));

        assert!(update_point(&conn, "k3", Some("k3"), "双链表", "").is_err());
        assert!(update_point(&conn, "missing", None, "双链表", "").is_err());
    }

    #[test]
    fn deleting_course_cascades_knowledge_points() {
        let conn = test_connection();
        seed_tree(&conn);

        conn.execute("DELETE FROM courses WHERE id = 'c1'", [])
            .expect("删除课程失败");

        let remaining = list_by_course(&conn, "c1").expect("读取知识点列表失败");
        assert!(remaining.is_empty());

        let c2_points = list_by_course(&conn, "c2").expect("读取知识点列表失败");
        assert_eq!(c2_points.len(), 1);
    }
}
