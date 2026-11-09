//! # 知识点层级路径工具
//!
//! 首页「最近创建的知识点」与全局搜索结果都需要展示知识点在课程树中的位置。
//! 这里统一加载知识点索引并回溯父节点，避免在多个命令里重复实现。

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use crate::db::db_error;

/// 知识点 ID -> (父知识点 ID, 标题)
pub type PointIndex = HashMap<String, (Option<String>, String)>;

/// 读取全部知识点的层级索引
pub fn load_point_index(conn: &Connection) -> Result<PointIndex, String> {
    let mut stmt = conn
        .prepare("SELECT id, parent_id, title FROM knowledge_points")
        .map_err(|e| db_error("读取知识点层级", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| db_error("读取知识点层级", e))?;

    let mut index = HashMap::new();
    for row in rows {
        let (id, parent_id, title) = row.map_err(|e| db_error("读取知识点层级", e))?;
        index.insert(id, (parent_id, title));
    }

    Ok(index)
}

/// 从指定知识点向上回溯，返回从顶层到该节点的标题路径（包含该节点）
///
/// 调用方传入父知识点 ID 时，得到的就是「根 → … → 父节点」的路径，
/// 用于在列表与搜索结果中展示知识点所属的层级。
/// 数据异常形成环时会自动停止，保证界面不会卡死。
pub fn path_titles(index: &PointIndex, start: Option<&str>) -> Vec<String> {
    let mut titles = Vec::new();
    let mut current = start.map(str::to_string);
    let mut visited: HashSet<String> = HashSet::new();

    while let Some(id) = current {
        if !visited.insert(id.clone()) {
            break;
        }

        let Some((parent_id, title)) = index.get(&id) else {
            break;
        };

        titles.push(title.clone());
        current = parent_id.clone();
    }

    titles.reverse();
    titles
}

/// 把路径拼成可直接展示的字符串，例如「线性表 / 顺序表」
pub fn join_path(titles: &[String], separator: &str) -> String {
    titles.join(separator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_titles_returns_root_to_target_order() {
        let mut index = PointIndex::new();
        index.insert("root".into(), (None, "根节点".into()));
        index.insert("child".into(), (Some("root".into()), "子节点".into()));
        index.insert("leaf".into(), (Some("child".into()), "叶子".into()));

        assert_eq!(
            path_titles(&index, Some("leaf")),
            vec!["根节点", "子节点", "叶子"]
        );
        assert_eq!(path_titles(&index, Some("child")), vec!["根节点", "子节点"]);
        assert!(path_titles(&index, None).is_empty());
        assert!(path_titles(&index, Some("missing")).is_empty());
    }

    #[test]
    fn path_titles_stops_on_cycles() {
        let mut index = PointIndex::new();
        index.insert("a".into(), (Some("b".into()), "A".into()));
        index.insert("b".into(), (Some("a".into()), "B".into()));

        assert_eq!(path_titles(&index, Some("a")).len(), 2);
    }

    #[test]
    fn join_path_keeps_display_order() {
        let titles = vec!["数据结构".to_string(), "线性表".to_string()];
        assert_eq!(join_path(&titles, " / "), "数据结构 / 线性表");
        assert_eq!(join_path(&[], " / "), "");
    }
}
