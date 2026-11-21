//! 课程包导出与导入
//!
//! 导出把一门课程下的全部内容聚合成单个 JSON 文件：
//! 课程信息、知识点层级、笔记正文与标签、课程内知识点关联，
//! 以及关联实际用到的关联类型定义。
//!
//! 导入时统一重新生成 UUID，并按名称复用已存在的标签与关联类型，
//! 因此课程包可以在不同数据库之间来回迁移；同名课程会自动重命名。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension, Row};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{db_error, get_connection};
use crate::models::course_transfer::{
    COURSE_PACKAGE_FORMAT, COURSE_PACKAGE_VERSION, CourseExportResult, CourseImportResult,
    CoursePackage, PackageCourse, PackageKnowledgePoint, PackageNote, PackageRelation,
    PackageRelationType,
};

/// 课程名称上限，与 create_course 的校验保持一致
const COURSE_NAME_LIMIT: usize = 60;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// 时间等字段为空时回退到当前时间，避免脏数据破坏排序
fn or_now(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

/// 没有扩展名时补上 `.json`；用户显式填写的其它扩展名保持原样
fn ensure_json_extension(path: PathBuf) -> PathBuf {
    if path.extension().is_none() {
        path.with_extension("json")
    } else {
        path
    }
}

/// 读取课程及其下全部内容，组装成课程包
fn load_package(conn: &Connection, course_id: &str) -> Result<CoursePackage, String> {
    let course = conn
        .query_row(
            "SELECT name, description, created_at, updated_at FROM courses WHERE id = ?1",
            [course_id],
            |row| {
                Ok(PackageCourse {
                    name: row.get(0)?,
                    description: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|e| db_error("读取课程", e))?
        .ok_or_else(|| "课程不存在或已被删除".to_string())?;

    let knowledge_points = {
        let mut stmt = conn
            .prepare(
                "SELECT id, parent_id, title, description, created_at, updated_at
                   FROM knowledge_points
                  WHERE course_id = ?1
                  ORDER BY created_at ASC, id ASC",
            )
            .map_err(|e| db_error("读取课程知识点", e))?;

        stmt.query_map([course_id], |row: &Row<'_>| {
            Ok(PackageKnowledgePoint {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| db_error("读取课程知识点", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取课程知识点", e))?
    };

    let mut notes = {
        let mut stmt = conn
            .prepare(
                "SELECT n.id, n.knowledge_point_id, n.title, n.content, n.created_at, n.updated_at
                   FROM notes n
                   JOIN knowledge_points kp ON n.knowledge_point_id = kp.id
                  WHERE kp.course_id = ?1
                  ORDER BY n.created_at ASC, n.id ASC",
            )
            .map_err(|e| db_error("读取课程笔记", e))?;

        stmt.query_map([course_id], |row: &Row<'_>| {
            Ok(PackageNote {
                id: row.get(0)?,
                knowledge_point_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                tags: Vec::new(),
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| db_error("读取课程笔记", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取课程笔记", e))?
    };

    // 一次性收集课程内所有笔记的标签，再回填到笔记上
    let mut tags_by_note: HashMap<String, Vec<String>> = HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT nt.note_id, t.name
                   FROM note_tags nt
                   JOIN tags t ON nt.tag_id = t.id
                   JOIN notes n ON nt.note_id = n.id
                   JOIN knowledge_points kp ON n.knowledge_point_id = kp.id
                  WHERE kp.course_id = ?1
                  ORDER BY t.name ASC",
            )
            .map_err(|e| db_error("读取课程标签", e))?;

        let rows = stmt
            .query_map([course_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| db_error("读取课程标签", e))?;

        for row in rows {
            let (note_id, tag_name) = row.map_err(|e| db_error("读取课程标签", e))?;
            tags_by_note.entry(note_id).or_default().push(tag_name);
        }
    }

    for note in &mut notes {
        if let Some(tags) = tags_by_note.remove(&note.id) {
            note.tags = tags;
        }
    }

    // 只导出两端都在本课程内的关联，避免导入到其它数据库后出现悬空端点
    let relations = {
        let mut stmt = conn
            .prepare(
                "SELECT kr.source_id, kr.target_id, kr.relation_type, kr.description, kr.created_at
                   FROM knowledge_relations kr
                   JOIN knowledge_points source ON kr.source_id = source.id
                   JOIN knowledge_points target ON kr.target_id = target.id
                  WHERE source.course_id = ?1 AND target.course_id = ?1
                  ORDER BY kr.created_at ASC, kr.id ASC",
            )
            .map_err(|e| db_error("读取课程关联", e))?;

        stmt.query_map([course_id], |row| {
            Ok(PackageRelation {
                source_id: row.get(0)?,
                target_id: row.get(1)?,
                relation_type: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| db_error("读取课程关联", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取课程关联", e))?
    };

    // 随包带上用到的关联类型定义（内置类型通常已存在，自定义类型会被补齐）
    let used_types: HashSet<&str> = relations
        .iter()
        .map(|relation| relation.relation_type.as_str())
        .collect();
    let relation_types = {
        let mut stmt = conn
            .prepare(
                "SELECT id, label, color, directed, description, sort_order
                   FROM relation_types",
            )
            .map_err(|e| db_error("读取关联类型", e))?;

        stmt.query_map([], |row: &Row<'_>| {
            Ok(PackageRelationType {
                id: row.get(0)?,
                label: row.get(1)?,
                color: row.get(2)?,
                directed: row.get::<_, i64>(3)? != 0,
                description: row.get(4)?,
                sort_order: row.get(5)?,
            })
        })
        .map_err(|e| db_error("读取关联类型", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取关联类型", e))?
        .into_iter()
        .filter(|relation_type| used_types.contains(relation_type.id.as_str()))
        .collect()
    };

    Ok(CoursePackage {
        format: COURSE_PACKAGE_FORMAT.to_string(),
        version: COURSE_PACKAGE_VERSION,
        exported_at: now(),
        course,
        knowledge_points,
        notes,
        relations,
        relation_types,
    })
}

/// 校验课程包结构；拒绝其它应用的 JSON 或引用不完整的包
fn validate_package(package: &CoursePackage) -> Result<(), String> {
    if package.format != COURSE_PACKAGE_FORMAT {
        return Err("所选文件不是本应用的课程包".into());
    }
    if package.version > COURSE_PACKAGE_VERSION {
        return Err("课程包版本高于当前应用支持的版本，请升级应用后再导入".into());
    }

    let name = package.course.name.trim();
    if name.is_empty() {
        return Err("课程包缺少课程名称".into());
    }
    if name.chars().count() > COURSE_NAME_LIMIT {
        return Err("课程包中的课程名称过长（最多 60 个字符）".into());
    }

    let mut point_ids: HashSet<&str> = HashSet::new();
    for point in &package.knowledge_points {
        if point.id.trim().is_empty() {
            return Err("课程包中的知识点缺少标识".into());
        }
        if point.title.trim().is_empty() {
            return Err("课程包中存在没有标题的知识点".into());
        }
        if !point_ids.insert(point.id.as_str()) {
            return Err("课程包中存在重复的知识点标识".into());
        }
    }

    for note in &package.notes {
        if !point_ids.contains(note.knowledge_point_id.as_str()) {
            return Err("课程包中的笔记指向了不存在的知识点".into());
        }
        if note.title.trim().is_empty() {
            return Err("课程包中存在没有标题的笔记".into());
        }
    }

    for relation in &package.relations {
        if !point_ids.contains(relation.source_id.as_str())
            || !point_ids.contains(relation.target_id.as_str())
        {
            return Err("课程包中的知识点关联指向了不存在的知识点".into());
        }
        if relation.source_id == relation.target_id {
            return Err("课程包中存在指向自身的知识点关联".into());
        }
    }

    Ok(())
}

/// 把课程包写入用户选择的路径
fn write_package(package: &CoursePackage, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.is_dir()
    {
        return Err("目标文件夹不存在，请重新选择保存位置".into());
    }

    let json = serde_json::to_string_pretty(package)
        .map_err(|e| format!("生成课程包失败：{e}"))?;
    fs::write(path, json).map_err(|e| format!("写入课程包失败：{e}"))
}

/// 读取并校验用户选择的课程包文件
fn read_package(path: &Path) -> Result<CoursePackage, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("读取课程包失败：{e}"))?;
    if !metadata.is_file() {
        return Err("请选择一个课程包文件".into());
    }
    if metadata.len() == 0 {
        return Err("课程包为空，无法导入".into());
    }

    let text = fs::read_to_string(path).map_err(|e| format!("读取课程包失败：{e}"))?;
    let package: CoursePackage =
        serde_json::from_str(&text).map_err(|e| format!("解析课程包失败：{e}"))?;

    validate_package(&package)?;

    Ok(package)
}

fn course_name_exists(conn: &Connection, name: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM courses WHERE name = ?1 COLLATE NOCASE",
            [name],
            |row| row.get(0),
        )
        .map_err(|e| db_error("校验课程名称", e))?;

    Ok(count > 0)
}

/// 在长度上限内追加后缀，避免重命名后超出课程名称限制
fn append_suffix(base: &str, suffix: &str) -> String {
    let max_base = COURSE_NAME_LIMIT.saturating_sub(suffix.chars().count());
    let mut name: String = base.chars().take(max_base).collect();
    name.push_str(suffix);
    name
}

/// 生成可用的课程名称：同名时依次尝试「课程名（导入）」「课程名（导入 2）」…
fn unique_course_name(conn: &Connection, base: &str) -> Result<(String, bool), String> {
    if !course_name_exists(conn, base)? {
        return Ok((base.to_string(), false));
    }

    for index in 1..=999 {
        let suffix = if index == 1 {
            "（导入）".to_string()
        } else {
            format!("（导入 {index}）")
        };
        let candidate = append_suffix(base, &suffix);

        if !course_name_exists(conn, &candidate)? {
            return Ok((candidate, true));
        }
    }

    Err("同名课程过多，无法自动命名，请先整理现有课程".into())
}

/// 按名称查找标签，不存在时创建；返回标签 ID
fn resolve_tag(conn: &Connection, name: &str) -> Result<String, String> {
    let existing: Option<String> = conn
        .query_row("SELECT id FROM tags WHERE name = ?1", [name], |row| row.get(0))
        .optional()
        .map_err(|e| db_error("读取标签", e))?;

    if let Some(id) = existing {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO tags (id, name) VALUES (?1, ?2)",
        params![id, name],
    )
    .map_err(|e| db_error("创建标签", e))?;

    Ok(id)
}

/// 在一个事务内写入课程包，任何一步失败都会整体回滚
fn import_package(conn: &mut Connection, package: CoursePackage) -> Result<CourseImportResult, String> {
    let timestamp = now();
    let tx = conn
        .transaction()
        .map_err(|e| db_error("开始导入事务", e))?;

    let base_name = package.course.name.trim();
    let (course_name, renamed) = unique_course_name(&tx, base_name)?;
    let course_id = Uuid::new_v4().to_string();

    tx.execute(
        "INSERT INTO courses (id, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            course_id,
            course_name,
            package.course.description.trim(),
            or_now(&package.course.created_at, &timestamp),
            or_now(&package.course.updated_at, &timestamp),
        ],
    )
    .map_err(|e| db_error("写入课程", e))?;

    // 旧知识点 ID → 新 UUID；先全部插入再回填 parent_id，避免依赖包内顺序
    let mut point_ids: HashMap<String, String> = HashMap::new();
    for point in &package.knowledge_points {
        point_ids.insert(point.id.clone(), Uuid::new_v4().to_string());
    }

    for point in &package.knowledge_points {
        tx.execute(
            "INSERT INTO knowledge_points
                 (id, course_id, parent_id, title, description, created_at, updated_at)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6)",
            params![
                point_ids[&point.id],
                course_id,
                point.title.trim(),
                point.description.trim(),
                or_now(&point.created_at, &timestamp),
                or_now(&point.updated_at, &timestamp),
            ],
        )
        .map_err(|e| db_error("写入知识点", e))?;
    }

    for point in &package.knowledge_points {
        let Some(parent_id) = point
            .parent_id
            .as_ref()
            .and_then(|parent| point_ids.get(parent))
        else {
            continue;
        };

        tx.execute(
            "UPDATE knowledge_points SET parent_id = ?1 WHERE id = ?2",
            params![parent_id, point_ids[&point.id]],
        )
        .map_err(|e| db_error("建立知识点层级", e))?;
    }

    let mut note_count = 0_i64;
    let mut tag_names: HashSet<String> = HashSet::new();

    for note in &package.notes {
        let note_id = Uuid::new_v4().to_string();

        tx.execute(
            "INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                note_id,
                point_ids[&note.knowledge_point_id],
                note.title.trim(),
                note.content,
                or_now(&note.created_at, &timestamp),
                or_now(&note.updated_at, &timestamp),
            ],
        )
        .map_err(|e| db_error("写入笔记", e))?;

        note_count += 1;

        for tag_name in &note.tags {
            let tag_name = tag_name.trim();
            if tag_name.is_empty() {
                continue;
            }

            tag_names.insert(tag_name.to_string());
            let tag_id = resolve_tag(&tx, tag_name)?;
            tx.execute(
                "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
                params![note_id, tag_id],
            )
            .map_err(|e| db_error("关联笔记标签", e))?;
        }
    }

    // 先补齐课程包携带的关联类型，再写入关联
    for relation_type in &package.relation_types {
        tx.execute(
            "INSERT OR IGNORE INTO relation_types
                 (id, label, color, directed, description, is_builtin, sort_order, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
            params![
                relation_type.id,
                relation_type.label,
                relation_type.color,
                i64::from(relation_type.directed),
                relation_type.description,
                relation_type.sort_order,
                timestamp,
            ],
        )
        .map_err(|e| db_error("写入关联类型", e))?;
    }

    let mut relation_count = 0_i64;
    for relation in &package.relations {
        let type_exists: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM relation_types WHERE id = ?1",
                [&relation.relation_type],
                |row| row.get(0),
            )
            .map_err(|e| db_error("校验关联类型", e))?;

        // 未知类型回退为「相关知识」，避免个别脏数据阻断整个导入
        let relation_type = if type_exists > 0 {
            relation.relation_type.clone()
        } else {
            "related".to_string()
        };

        let changed = tx
            .execute(
                "INSERT OR IGNORE INTO knowledge_relations
                     (id, source_id, target_id, relation_type, description, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    Uuid::new_v4().to_string(),
                    point_ids[&relation.source_id],
                    point_ids[&relation.target_id],
                    relation_type,
                    relation.description.trim(),
                    or_now(&relation.created_at, &timestamp),
                ],
            )
            .map_err(|e| db_error("写入知识点关联", e))?;

        relation_count += changed as i64;
    }

    tx.commit().map_err(|e| db_error("提交导入事务", e))?;

    Ok(CourseImportResult {
        course_id,
        course_name,
        renamed,
        knowledge_point_count: package.knowledge_points.len() as i64,
        note_count,
        relation_count,
        tag_count: tag_names.len() as i64,
    })
}

/// 把课程及其下全部内容导出为单个 JSON 课程包
#[tauri::command]
pub fn export_course_package(
    app: AppHandle,
    course_id: String,
    destination: String,
) -> Result<CourseExportResult, String> {
    let destination = destination.trim();
    if destination.is_empty() {
        return Err("请选择课程包的保存位置".into());
    }

    let conn = get_connection(&app)?;
    let package = load_package(&conn, &course_id)?;
    let path = ensure_json_extension(PathBuf::from(destination));

    write_package(&package, &path)?;

    let tag_count = package
        .notes
        .iter()
        .flat_map(|note| note.tags.iter())
        .collect::<HashSet<_>>()
        .len() as i64;

    Ok(CourseExportResult {
        path: path.to_string_lossy().into_owned(),
        course_name: package.course.name.clone(),
        knowledge_point_count: package.knowledge_points.len() as i64,
        note_count: package.notes.len() as i64,
        relation_count: package.relations.len() as i64,
        tag_count,
    })
}

/// 从 JSON 课程包导入课程；同名课程自动重命名
#[tauri::command]
pub fn import_course_package(app: AppHandle, source: String) -> Result<CourseImportResult, String> {
    let source = source.trim();
    if source.is_empty() {
        return Err("请选择要导入的课程包".into());
    }

    let package = read_package(Path::new(source))?;
    let mut conn = get_connection(&app)?;

    import_package(&mut conn, package)
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

    fn seed_course(conn: &Connection) {
        conn.execute_batch(
            "
            INSERT INTO courses (id, name, description, created_at, updated_at)
            VALUES ('c1', '数据结构', '课程描述', '2026-01-01T00:00:00Z', '2026-01-02T00:00:00Z');

            INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at)
            VALUES ('kp1', 'c1', NULL, '线性结构', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
                   ('kp2', 'c1', 'kp1', '链表', '链表说明', '2026-01-01T01:00:00Z', '2026-01-01T01:00:00Z');

            INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at)
            VALUES ('n1', 'kp1', '线性表', '正文 A', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
                   ('n2', 'kp2', '链表操作', '正文 B', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');

            INSERT INTO tags (id, name) VALUES ('t1', '算法');
            INSERT INTO note_tags (note_id, tag_id) VALUES ('n2', 't1');

            INSERT INTO knowledge_relations
                (id, source_id, target_id, relation_type, description, created_at)
            VALUES ('r1', 'kp1', 'kp2', 'contains', '包含', '2026-01-01T00:00:00Z');
            ",
        )
        .expect("写入测试数据失败");
    }

    fn count_rows(conn: &Connection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0))
            .expect("统计数据失败")
    }

    #[test]
    fn export_and_import_round_trip() {
        let mut conn = test_connection();
        seed_course(&conn);

        let package = load_package(&conn, "c1").expect("导出课程包失败");
        assert_eq!(package.knowledge_points.len(), 2);
        assert_eq!(package.notes.len(), 2);
        assert_eq!(package.relations.len(), 1);
        assert!(
            package
                .relation_types
                .iter()
                .any(|relation_type| relation_type.id == "contains")
        );

        let tagged = package
            .notes
            .iter()
            .find(|note| note.id == "n2")
            .expect("应包含链表笔记");
        assert_eq!(tagged.tags, vec!["算法"]);

        let json = serde_json::to_string_pretty(&package).expect("序列化课程包失败");
        let parsed: CoursePackage = serde_json::from_str(&json).expect("反序列化课程包失败");
        let result = import_package(&mut conn, parsed).expect("导入课程包失败");

        assert!(result.renamed);
        assert_eq!(result.course_name, "数据结构（导入）");
        assert_eq!(result.knowledge_point_count, 2);
        assert_eq!(result.note_count, 2);
        assert_eq!(result.relation_count, 1);
        assert_eq!(result.tag_count, 1);
        assert_eq!(count_rows(&conn, "courses"), 2);

        // 标签按名称复用，不应产生重复的「算法」
        let tag_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tags WHERE name = '算法'",
                [],
                |row| row.get(0),
            )
            .expect("统计标签失败");
        assert_eq!(tag_count, 1);

        // 知识点层级应保持：链表 仍是 线性结构 的子节点
        let hierarchy: i64 = conn
            .query_row(
                "SELECT COUNT(*)
                   FROM knowledge_points child
                   JOIN knowledge_points parent ON child.parent_id = parent.id
                  WHERE child.course_id = ?1
                    AND child.title = '链表'
                    AND parent.title = '线性结构'",
                [&result.course_id],
                |row| row.get(0),
            )
            .expect("校验知识点层级失败");
        assert_eq!(hierarchy, 1);

        // 笔记正文应完整保留
        let content: String = conn
            .query_row(
                "SELECT n.content
                   FROM notes n
                   JOIN knowledge_points kp ON n.knowledge_point_id = kp.id
                  WHERE kp.course_id = ?1 AND n.title = '链表操作'",
                [&result.course_id],
                |row| row.get(0),
            )
            .expect("读取导入笔记失败");
        assert_eq!(content, "正文 B");
    }

    #[test]
    fn repeated_import_renames_with_index() {
        let mut conn = test_connection();
        seed_course(&conn);

        let package = load_package(&conn, "c1").expect("导出课程包失败");
        let json = serde_json::to_string(&package).expect("序列化课程包失败");

        let first = import_package(&mut conn, serde_json::from_str(&json).unwrap())
            .expect("第一次导入失败");
        let second = import_package(&mut conn, serde_json::from_str(&json).unwrap())
            .expect("第二次导入失败");

        assert_eq!(first.course_name, "数据结构（导入）");
        assert_eq!(second.course_name, "数据结构（导入 2）");
        assert_eq!(count_rows(&conn, "courses"), 3);
    }

    #[test]
    fn cross_course_relations_are_not_exported() {
        let conn = test_connection();
        seed_course(&conn);
        conn.execute_batch(
            "
            INSERT INTO courses (id, name, description, created_at, updated_at)
            VALUES ('c2', '操作系统', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');

            INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at)
            VALUES ('kp3', 'c2', NULL, '进程', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');

            INSERT INTO knowledge_relations
                (id, source_id, target_id, relation_type, description, created_at)
            VALUES ('r2', 'kp1', 'kp3', 'related', '跨课程', '2026-01-01T00:00:00Z');
            ",
        )
        .expect("写入跨课程数据失败");

        let package = load_package(&conn, "c1").expect("导出课程包失败");
        assert_eq!(package.relations.len(), 1, "只应导出课程内关联");
        assert_eq!(package.relations[0].relation_type, "contains");
    }

    #[test]
    fn invalid_packages_are_rejected() {
        let conn = test_connection();
        seed_course(&conn);
        let package = load_package(&conn, "c1").expect("导出课程包失败");

        let mut foreign = package.clone();
        foreign.format = "other-app".into();
        assert!(validate_package(&foreign).is_err());

        let mut dangling_note = package.clone();
        dangling_note.notes[0].knowledge_point_id = "missing".into();
        assert!(validate_package(&dangling_note).is_err());

        let mut dangling_relation = package.clone();
        dangling_relation.relations[0].target_id = "missing".into();
        assert!(validate_package(&dangling_relation).is_err());

        let mut self_relation = package.clone();
        self_relation.relations[0].target_id = self_relation.relations[0].source_id.clone();
        assert!(validate_package(&self_relation).is_err());
    }

    #[test]
    fn package_file_round_trip_preserves_unicode() {
        let conn = test_connection();
        seed_course(&conn);
        let package = load_package(&conn, "c1").expect("导出课程包失败");

        let dir = std::env::temp_dir().join(format!("kgn-course-package-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("创建临时目录失败");
        let path = dir.join("课程包.json");

        write_package(&package, &path).expect("写入课程包失败");
        let loaded = read_package(&path).expect("读取课程包失败");

        assert_eq!(loaded.course.name, "数据结构");
        assert_eq!(loaded.notes[1].content, "正文 B");
        assert_eq!(loaded.notes[1].tags, vec!["算法"]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_keeps_course_name_within_limit() {
        let long_name = "课".repeat(COURSE_NAME_LIMIT);
        let renamed = append_suffix(&long_name, "（导入）");

        assert!(renamed.chars().count() <= COURSE_NAME_LIMIT);
        assert!(renamed.ends_with("（导入）"));
    }
}