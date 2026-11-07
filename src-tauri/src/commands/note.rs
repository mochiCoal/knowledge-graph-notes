use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension, Row};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{db_error, escape_like, get_connection};
use crate::models::note::{MarkdownFile, Note, NoteSummary};

/// 笔记标题最大长度（与前端输入框限制一致）
const NOTE_TITLE_MAX: usize = 200;
/// 列表摘要的最大字符数
const EXCERPT_MAX: usize = 120;
/// 允许导入的 Markdown 文件大小上限（2 MB）
const MAX_MARKDOWN_BYTES: u64 = 2 * 1024 * 1024;

const NOTE_SELECT: &str = "
SELECT id, knowledge_point_id, title, content, created_at, updated_at
FROM notes
";

/// 笔记列表 / 搜索统一使用的 SELECT 片段，额外带出所属知识点与课程。
const NOTE_SUMMARY_SELECT: &str = "
SELECT n.id, n.knowledge_point_id, n.title, n.content, n.created_at, n.updated_at,
       kp.title, kp.course_id, c.name
FROM notes n
JOIN knowledge_points kp ON kp.id = n.knowledge_point_id
JOIN courses c ON c.id = kp.course_id
";

fn map_note(row: &Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        knowledge_point_id: row.get(1)?,
        title: row.get(2)?,
        content: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn map_note_summary(row: &Row<'_>) -> rusqlite::Result<NoteSummary> {
    let content: String = row.get(3)?;

    Ok(NoteSummary {
        id: row.get(0)?,
        knowledge_point_id: row.get(1)?,
        title: row.get(2)?,
        excerpt: excerpt(&content, EXCERPT_MAX),
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        knowledge_point_title: row.get(6)?,
        course_id: row.get(7)?,
        course_name: row.get(8)?,
        character_count: count_characters(&content),
        tag_names: Vec::new(),
    })
}

/// 统计正文字符数（忽略空白，与前端 `countCharacters` 保持一致）
fn count_characters(text: &str) -> i64 {
    text.chars().filter(|ch| !ch.is_whitespace()).count() as i64
}

fn truncate_chars(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }

    let mut truncated: String = text.chars().take(limit).collect();
    truncated.push('…');
    truncated
}

/// 生成列表摘要：去掉常见 Markdown 标记、折叠空白并截断
fn excerpt(content: &str, limit: usize) -> String {
    let mut buffer = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 纯分隔线（--- / *** / ___）没有摘要价值
        if trimmed.chars().count() >= 3
            && trimmed
                .chars()
                .all(|ch| matches!(ch, '-' | '*' | '_' | '='))
        {
            continue;
        }

        let stripped = trimmed.trim_start_matches(|ch: char| {
            matches!(ch, '#' | '>' | '-' | '*' | '+' | '|' | '`' | ' ')
        });
        let cleaned: String = stripped
            .chars()
            .filter(|ch| !matches!(ch, '`' | '*'))
            .collect();
        let cleaned = cleaned.trim();
        if cleaned.is_empty() {
            continue;
        }

        if !buffer.is_empty() {
            buffer.push(' ');
        }
        buffer.push_str(cleaned);

        if buffer.chars().count() >= limit * 2 {
            break;
        }
    }

    truncate_chars(&buffer, limit)
}

fn query_summaries(
    conn: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
) -> Result<Vec<NoteSummary>, String> {
    let mut summaries = {
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| db_error("读取笔记列表", e))?;

        stmt.query_map(params, map_note_summary)
            .map_err(|e| db_error("读取笔记列表", e))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| db_error("读取笔记列表", e))?
    };

    attach_tag_names(conn, &mut summaries)?;
    Ok(summaries)
}

/// 为笔记摘要补充标签名称（按名称排序）
fn attach_tag_names(conn: &Connection, summaries: &mut [NoteSummary]) -> Result<(), String> {
    if summaries.is_empty() {
        return Ok(());
    }

    let ids: HashSet<String> = summaries
        .iter()
        .map(|summary| summary.id.clone())
        .collect();

    let mut stmt = conn
        .prepare(
            "SELECT nt.note_id, t.name
             FROM note_tags nt
             INNER JOIN tags t ON t.id = nt.tag_id",
        )
        .map_err(|e| db_error("读取笔记标签", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| db_error("读取笔记标签", e))?;

    let mut tag_names: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let (note_id, name) = row.map_err(|e| db_error("读取笔记标签", e))?;
        if ids.contains(&note_id) {
            tag_names.entry(note_id).or_default().push(name);
        }
    }

    for summary in summaries.iter_mut() {
        let mut names = tag_names.remove(&summary.id).unwrap_or_default();
        names.sort();
        summary.tag_names = names;
    }

    Ok(())
}

/// 查询笔记摘要列表；`keyword` 与 `tag_id` 均可选，同时提供时取交集
fn list_summaries(
    conn: &Connection,
    keyword: Option<&str>,
    tag_id: Option<&str>,
) -> Result<Vec<NoteSummary>, String> {
    let keyword = keyword.map(str::trim).filter(|keyword| !keyword.is_empty());
    let tag_id = tag_id.map(str::trim).filter(|tag_id| !tag_id.is_empty());

    let mut clauses: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    if let Some(keyword) = keyword {
        let index = values.len() + 1;
        clauses.push(format!(
            "(n.title LIKE ?{index} ESCAPE '\\' OR n.content LIKE ?{index} ESCAPE '\\')"
        ));
        values.push(format!("%{}%", escape_like(keyword)));
    }

    if let Some(tag_id) = tag_id {
        let index = values.len() + 1;
        clauses.push(format!(
            "EXISTS (SELECT 1 FROM note_tags nt WHERE nt.note_id = n.id AND nt.tag_id = ?{index})"
        ));
        values.push(tag_id.to_string());
    }

    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };

    // 标题命中的笔记排在内容命中之前
    let order_sql = if keyword.is_some() {
        "ORDER BY CASE WHEN n.title LIKE ?1 ESCAPE '\\' THEN 0 ELSE 1 END, n.updated_at DESC"
    } else {
        "ORDER BY n.updated_at DESC"
    };

    let sql = format!("{NOTE_SUMMARY_SELECT} {where_sql} {order_sql}");

    query_summaries(conn, &sql, rusqlite::params_from_iter(values.iter().map(String::as_str)))
}

fn point_title(conn: &Connection, knowledge_point_id: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT title FROM knowledge_points WHERE id = ?1",
        [knowledge_point_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| db_error("读取知识点", e))?
    .ok_or_else(|| "知识点不存在，无法保存笔记".to_string())
}

fn load_note(conn: &Connection, note_id: &str) -> Result<Note, String> {
    let sql = format!("{NOTE_SELECT} WHERE id = ?1");
    conn.query_row(&sql, [note_id], map_note)
        .optional()
        .map_err(|e| db_error("读取笔记", e))?
        .ok_or_else(|| "笔记不存在或已被删除，请刷新后重试".to_string())
}

/// 导出 Markdown 时的文件内容：正文没有一级标题时自动补上笔记标题
fn compose_markdown(note: &Note) -> String {
    let content = note.content.as_str();
    let starts_with_heading = content
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim_start().starts_with("# "))
        .unwrap_or(false);

    if starts_with_heading {
        format!("{}\n", content.trim_end())
    } else if content.trim().is_empty() {
        format!("# {}\n", note.title)
    } else {
        format!("# {}\n\n{}\n", note.title, content.trim_end())
    }
}

fn write_markdown(note: &Note, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        return Err("目标文件夹不存在，请重新选择保存位置".into());
    }

    std::fs::write(path, compose_markdown(note))
        .map_err(|e| format!("写入 Markdown 文件失败：{e}"))
}

/// 读取并解析 Markdown 文件：首行一级标题作为笔记标题，其余内容作为正文
fn read_markdown(path: &Path) -> Result<MarkdownFile, String> {
    let metadata = std::fs::metadata(path).map_err(|e| format!("读取文件失败：{e}"))?;
    if !metadata.is_file() {
        return Err("请选择一个 Markdown 文件".into());
    }
    if metadata.len() > MAX_MARKDOWN_BYTES {
        return Err(format!(
            "文件过大（超过 {} MB），请选择更小的 Markdown 文件",
            MAX_MARKDOWN_BYTES / 1024 / 1024
        ));
    }

    let bytes = std::fs::read(path).map_err(|e| format!("读取文件失败：{e}"))?;
    let text =
        String::from_utf8(bytes).map_err(|_| "文件不是有效的 UTF-8 文本，无法导入".to_string())?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("未命名.md")
        .to_string();
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("未命名");

    let mut lines: Vec<&str> = text.lines().collect();
    let mut title = truncate_chars(stem, NOTE_TITLE_MAX);

    // 首个非空行若是一级标题，则提取为标题并从正文中移除，避免标题重复
    if let Some(index) = lines.iter().position(|line| !line.trim().is_empty()) {
        let candidate = lines[index].trim();
        if let Some(rest) = candidate.strip_prefix("# ") {
            let heading = rest.trim();
            if !heading.is_empty() {
                title = truncate_chars(heading, NOTE_TITLE_MAX);
                lines.drain(..=index);
            }
        }
    }

    while lines
        .first()
        .map(|line| line.trim().is_empty())
        .unwrap_or(false)
    {
        lines.remove(0);
    }

    let content = lines.join("\n");

    Ok(MarkdownFile {
        file_name,
        title,
        character_count: count_characters(&content),
        content,
    })
}

fn ensure_markdown_extension(path: PathBuf) -> PathBuf {
    if path.extension().is_none() {
        path.with_extension("md")
    } else {
        path
    }
}

/// 查询某个知识点最近编辑的一条笔记
#[tauri::command]
pub fn get_note(app: AppHandle, knowledge_point_id: String) -> Result<Option<Note>, String> {
    let conn = get_connection(&app)?;

    conn.query_row(
        "SELECT id, knowledge_point_id, title, content, created_at, updated_at
         FROM notes
         WHERE knowledge_point_id = ?1
         ORDER BY updated_at DESC
         LIMIT 1",
        [&knowledge_point_id],
        map_note,
    )
    .optional()
    .map_err(|e| db_error("读取笔记", e))
}

/// 保存笔记：note_id 为空时新建（若该知识点已有笔记则复用最近一条）
#[tauri::command]
pub fn save_note(
    app: AppHandle,
    note_id: Option<String>,
    knowledge_point_id: String,
    title: String,
    content: String,
) -> Result<Note, String> {
    let conn = get_connection(&app)?;
    let now = chrono::Utc::now().to_rfc3339();

    let default_title = point_title(&conn, &knowledge_point_id)?;
    let mut note_title = title.trim().to_string();
    if note_title.is_empty() {
        note_title = default_title;
    }
    if note_title.chars().count() > NOTE_TITLE_MAX {
        return Err(format!("笔记标题不能超过 {NOTE_TITLE_MAX} 个字符"));
    }

    if let Some(existing_id) = note_id {
        let affected = conn
            .execute(
                "UPDATE notes
                 SET title = ?1, content = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![note_title, content, now, existing_id],
            )
            .map_err(|e| db_error("保存笔记", e))?;

        if affected == 0 {
            return Err("笔记不存在或已被删除，请刷新后重试".into());
        }

        return load_note(&conn, &existing_id);
    }

    // 未提供 note_id 时，优先复用该知识点已有的笔记，避免重复创建
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM notes
             WHERE knowledge_point_id = ?1
             ORDER BY updated_at DESC
             LIMIT 1",
            [&knowledge_point_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| db_error("读取笔记", e))?;

    if let Some(id) = existing_id {
        conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
            params![note_title, content, now, id],
        )
        .map_err(|e| db_error("保存笔记", e))?;
        return load_note(&conn, &id);
    }

    let note = Note {
        id: Uuid::new_v4().to_string(),
        knowledge_point_id,
        title: note_title,
        content,
        created_at: now.clone(),
        updated_at: now,
    };

    conn.execute(
        "INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            note.id,
            note.knowledge_point_id,
            note.title,
            note.content,
            note.created_at,
            note.updated_at
        ],
    )
    .map_err(|e| db_error("创建笔记", e))?;

    Ok(note)
}

/// 删除笔记
#[tauri::command]
pub fn delete_note(app: AppHandle, note_id: String) -> Result<(), String> {
    let conn = get_connection(&app)?;
    conn.execute("DELETE FROM notes WHERE id = ?1", [&note_id])
        .map_err(|e| db_error("删除笔记", e))?;
    Ok(())
}

/// 查询全部笔记（笔记库列表，按更新时间倒序，可按标签筛选）
#[tauri::command]
pub fn list_notes(app: AppHandle, tag_id: Option<String>) -> Result<Vec<NoteSummary>, String> {
    let conn = get_connection(&app)?;
    list_summaries(&conn, None, tag_id.as_deref())
}

/// 按关键字搜索笔记标题与正文（空关键字等价于查询全部，可按标签筛选）
#[tauri::command]
pub fn search_notes(
    app: AppHandle,
    keyword: String,
    tag_id: Option<String>,
) -> Result<Vec<NoteSummary>, String> {
    let conn = get_connection(&app)?;
    list_summaries(&conn, Some(&keyword), tag_id.as_deref())
}

/// 把笔记导出为 Markdown 文件，返回实际写入的路径
#[tauri::command]
pub fn export_note_markdown(
    app: AppHandle,
    note_id: String,
    file_path: String,
) -> Result<String, String> {
    let conn = get_connection(&app)?;
    let note = load_note(&conn, &note_id)?;
    let path = ensure_markdown_extension(PathBuf::from(&file_path));

    write_markdown(&note, &path)?;

    Ok(path.to_string_lossy().into_owned())
}

/// 读取本地 Markdown 文件（导入前预览，标题取首行一级标题或文件名）
#[tauri::command]
pub fn read_markdown_file(file_path: String) -> Result<MarkdownFile, String> {
    read_markdown(Path::new(&file_path))
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

    fn seed(conn: &Connection) {
        conn.execute(
            "INSERT INTO courses (id, name, description, created_at, updated_at)
             VALUES ('c1', '数据结构', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        )
        .expect("插入课程失败");
        conn.execute(
            "INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at)
             VALUES ('kp1', 'c1', NULL, '链表', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        )
        .expect("插入知识点失败");
        conn.execute(
            "INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at)
             VALUES ('kp2', 'c1', NULL, '排序算法', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        )
        .expect("插入知识点失败");

        conn.execute(
            "INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at)
             VALUES ('n1', 'kp1', '链表笔记', '# 链表\n\n**单链表** 插入 O(1)。\n- 双向链表', '2026-01-01T00:00:00Z', '2026-01-02T00:00:00Z')",
            [],
        )
        .expect("插入笔记失败");
        conn.execute(
            "INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at)
             VALUES ('n2', 'kp2', '快速排序', '分治思想，平均 O(n log n)。', '2026-01-01T00:00:00Z', '2026-01-03T00:00:00Z')",
            [],
        )
        .expect("插入笔记失败");
    }

    #[test]
    fn excerpt_strips_markdown_and_truncates() {
        let text = excerpt("# 标题\n\n**重点** 内容 `code`\n\n---\n第二段", 80);
        assert!(!text.contains('#'));
        assert!(!text.contains('*'));
        assert!(!text.contains('`'));
        assert!(text.contains("重点"));
        assert!(text.contains("第二段"));

        let long = excerpt(&"字".repeat(300), 20);
        assert_eq!(long.chars().count(), 21);
        assert!(long.ends_with('…'));

        assert_eq!(excerpt("   \n\n", 20), "");
    }

    #[test]
    fn escape_like_treats_wildcards_literally() {
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("c\\d"), "c\\\\d");
        assert_eq!(escape_like("普通"), "普通");
    }

    #[test]
    fn summaries_are_ordered_and_joined_with_point_and_course() {
        let conn = test_connection();
        seed(&conn);

        let summaries = list_summaries(&conn, None, None).expect("读取笔记列表失败");
        assert_eq!(
            summaries
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["n2", "n1"]
        );

        let first = &summaries[0];
        assert_eq!(first.knowledge_point_title, "排序算法");
        assert_eq!(first.course_name, "数据结构");
        assert_eq!(first.course_id, "c1");
        assert_eq!(first.character_count, 16, "忽略空白后的字符数");
        assert!(!first.excerpt.is_empty());
        assert!(first.tag_names.is_empty());
    }

    #[test]
    fn search_matches_title_and_content() {
        let conn = test_connection();
        seed(&conn);

        let by_title = list_summaries(&conn, Some("链表"), None).expect("搜索笔记失败");
        assert_eq!(by_title.len(), 1);
        assert_eq!(by_title[0].id, "n1");

        let by_content = list_summaries(&conn, Some("分治"), None).expect("搜索笔记失败");
        assert_eq!(by_content.len(), 1);
        assert_eq!(by_content[0].id, "n2");

        // 标题命中的笔记排在内容命中的前面
        let both = list_summaries(&conn, Some("排序"), None).expect("搜索笔记失败");
        assert_eq!(both[0].id, "n2");

        assert!(
            list_summaries(&conn, Some("不存在的关键字"), None)
                .expect("搜索笔记失败")
                .is_empty()
        );
        // 通配符按字面量处理，不会匹配到所有笔记
        assert!(
            list_summaries(&conn, Some("%"), None)
                .expect("搜索笔记失败")
                .is_empty()
        );
        // 空白关键字等价于查询全部
        assert_eq!(
            list_summaries(&conn, Some("   "), None)
                .expect("搜索笔记失败")
                .len(),
            2
        );
    }

    #[test]
    fn summaries_include_tags_and_support_tag_filter() {
        let conn = test_connection();
        seed(&conn);

        conn.execute("INSERT INTO tags (id, name) VALUES ('tg1', '重点')", [])
            .expect("插入标签失败");
        conn.execute("INSERT INTO tags (id, name) VALUES ('tg2', '面试')", [])
            .expect("插入标签失败");
        for (note_id, tag_id) in [("n1", "tg1"), ("n1", "tg2"), ("n2", "tg1")] {
            conn.execute(
                "INSERT INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
                params![note_id, tag_id],
            )
            .expect("插入笔记标签失败");
        }

        let all = list_summaries(&conn, None, None).expect("读取笔记列表失败");
        let n1 = all.iter().find(|item| item.id == "n1").expect("缺少 n1");
        assert_eq!(n1.tag_names, vec!["重点", "面试"]);
        let n2 = all.iter().find(|item| item.id == "n2").expect("缺少 n2");
        assert_eq!(n2.tag_names, vec!["重点"]);

        let by_tag = list_summaries(&conn, None, Some("tg2")).expect("按标签筛选失败");
        assert_eq!(by_tag.len(), 1);
        assert_eq!(by_tag[0].id, "n1");

        // 关键字与标签同时生效
        let combined = list_summaries(&conn, Some("链表"), Some("tg1")).expect("组合筛选失败");
        assert_eq!(combined.len(), 1);
        assert_eq!(combined[0].id, "n1");

        assert!(
            list_summaries(&conn, Some("分治"), Some("tg2"))
                .expect("组合筛选失败")
                .is_empty()
        );
        // 不存在的标签不会匹配任何笔记
        assert!(
            list_summaries(&conn, None, Some("missing"))
                .expect("按标签筛选失败")
                .is_empty()
        );
    }

    #[test]
    fn compose_markdown_adds_title_only_when_missing() {
        let mut note = Note {
            id: "n1".into(),
            knowledge_point_id: "kp1".into(),
            title: "链表笔记".into(),
            content: "正文内容".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        assert_eq!(compose_markdown(&note), "# 链表笔记\n\n正文内容\n");

        note.content = "# 已有标题\n\n正文".into();
        assert_eq!(compose_markdown(&note), "# 已有标题\n\n正文\n");

        note.content = "   ".into();
        assert_eq!(compose_markdown(&note), "# 链表笔记\n");
    }

    #[test]
    fn markdown_file_round_trip_uses_h1_as_title() {
        let conn = test_connection();
        seed(&conn);
        let note = load_note(&conn, "n1").expect("读取笔记失败");

        let path = std::env::temp_dir().join(format!("kgn-note-{}.md", Uuid::new_v4()));
        write_markdown(&note, &path).expect("导出 Markdown 失败");

        // 正文本身已有一级标题时优先使用它，不重复写入笔记标题
        let file = read_markdown(&path).expect("读取 Markdown 失败");
        assert_eq!(file.title, "链表");
        assert!(file.content.contains("双向链表"));
        assert!(!file.content.starts_with("# 链表"));

        // 正文没有一级标题时，导出的文件会把笔记标题写成一级标题
        let plain = Note {
            id: "n3".into(),
            knowledge_point_id: "kp1".into(),
            title: "新标题".into(),
            content: "只有正文".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        write_markdown(&plain, &path).expect("导出 Markdown 失败");

        let file = read_markdown(&path).expect("读取 Markdown 失败");
        assert_eq!(file.title, "新标题");
        assert_eq!(file.content, "只有正文");
        assert!(file.file_name.ends_with(".md"));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn read_markdown_falls_back_to_file_name() {
        let path = std::env::temp_dir().join(format!("kgn-import-{}.md", Uuid::new_v4()));
        std::fs::write(&path, "没有一级标题的正文").expect("写入临时文件失败");

        let file = read_markdown(&path).expect("读取 Markdown 失败");
        assert!(file.title.starts_with("kgn-import-"));
        assert_eq!(file.content, "没有一级标题的正文");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn read_markdown_rejects_invalid_utf8() {
        let path = std::env::temp_dir().join(format!("kgn-bad-{}.md", Uuid::new_v4()));
        std::fs::write(&path, [0xff, 0xfe, 0xfd]).expect("写入临时文件失败");

        assert!(read_markdown(&path).is_err());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn ensure_markdown_extension_appends_md_only_when_needed() {
        assert_eq!(
            ensure_markdown_extension(PathBuf::from("C:/tmp/笔记")),
            PathBuf::from("C:/tmp/笔记.md")
        );
        assert_eq!(
            ensure_markdown_extension(PathBuf::from("C:/tmp/笔记.markdown")),
            PathBuf::from("C:/tmp/笔记.markdown")
        );
    }
}
