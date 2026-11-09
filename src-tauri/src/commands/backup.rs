use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{
    database_path, db_error, get_connection, initialize_database, open_connection, table_counts,
};
use crate::models::backup::{BackupResult, DatabaseInfo, RestoreResult};

/// 恢复前必须存在的业务表；缺少任何一张都说明备份不属于本应用
const REQUIRED_TABLES: &[&str] = &[
    "courses",
    "knowledge_points",
    "notes",
    "tags",
    "note_tags",
    "knowledge_relations",
];

/// SQLite 文件头（前 16 字节）
const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";

/// Windows 下数据库文件可能被杀毒软件 / 索引服务短暂占用，复制与替换都做少量重试
const IO_ATTEMPTS: usize = 6;
const IO_RETRY_DELAY: Duration = Duration::from_millis(120);

fn path_display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn file_size(path: &Path) -> Result<u64, String> {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|e| format!("读取文件信息失败：{e}"))
}

fn modified_at(path: &Path) -> Option<String> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let datetime: chrono::DateTime<chrono::Local> = modified.into();
    Some(datetime.to_rfc3339())
}

/// 判断两个路径是否指向同一个文件（用于阻止把备份写回正在使用的数据库）
fn same_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn ensure_parent_directory(path: &Path) -> Result<(), String> {
    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };

    if !parent.is_dir() {
        return Err("目标文件夹不存在，请重新选择保存位置".into());
    }

    Ok(())
}

/// 没有扩展名时补上 `.db`；用户显式填写的其它扩展名保持原样
fn ensure_database_extension(path: PathBuf) -> PathBuf {
    if path.extension().is_none() {
        path.with_extension("db")
    } else {
        path
    }
}

/// 校验一个文件是否为本应用可用的 SQLite 数据库
fn validate_database_file(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|e| format!("读取备份文件失败：{e}"))?;
    if !metadata.is_file() {
        return Err("请选择一个数据库文件".into());
    }
    if metadata.len() < SQLITE_HEADER.len() as u64 {
        return Err("备份文件为空或过小，无法恢复".into());
    }

    let mut header = [0_u8; 16];
    let mut file = fs::File::open(path).map_err(|e| format!("读取备份文件失败：{e}"))?;
    file.read_exact(&mut header)
        .map_err(|_| "读取备份文件失败：文件不完整".to_string())?;
    if &header != SQLITE_HEADER {
        return Err("所选文件不是有效的 SQLite 数据库".into());
    }

    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("打开备份文件失败：{e}"))?;

    let check: String = conn
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|e| format!("校验备份文件失败：{e}"))?;
    if check != "ok" {
        return Err("备份文件已损坏，无法恢复".into());
    }

    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
        .map_err(|e| db_error("读取备份文件结构", e))?;
    let table_names: HashSet<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| db_error("读取备份文件结构", e))?
        .collect::<rusqlite::Result<HashSet<_>>>()
        .map_err(|e| db_error("读取备份文件结构", e))?;

    let missing: Vec<&str> = REQUIRED_TABLES
        .iter()
        .copied()
        .filter(|table| !table_names.contains(*table))
        .collect();

    if !missing.is_empty() {
        return Err(format!(
            "所选数据库缺少必要的表（{}），可能不是本应用的备份",
            missing.join("、")
        ));
    }

    Ok(())
}

/// 读取数据库信息（路径、大小、修改时间、各表数据量）
fn collect_database_info(path: &Path) -> Result<DatabaseInfo, String> {
    let conn = open_connection(path)?;
    let counts = table_counts(&conn)?;

    Ok(DatabaseInfo {
        path: path_display(path),
        directory: path
            .parent()
            .map(path_display)
            .unwrap_or_else(|| path_display(path)),
        size_bytes: file_size(path)?,
        modified_at: modified_at(path),
        sqlite_version: rusqlite::version().to_string(),
        counts,
    })
}

fn copy_with_retry(from: &Path, to: &Path) -> std::io::Result<u64> {
    let mut last_error = None;

    for attempt in 0..IO_ATTEMPTS {
        match fs::copy(from, to) {
            Ok(bytes) => return Ok(bytes),
            Err(error) => {
                last_error = Some(error);
                if attempt + 1 < IO_ATTEMPTS {
                    sleep(IO_RETRY_DELAY);
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| std::io::Error::other("复制文件失败")))
}

fn remove_file_with_retry(path: &Path) -> std::io::Result<()> {
    let mut last_error = None;

    for attempt in 0..IO_ATTEMPTS {
        match fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                if attempt + 1 < IO_ATTEMPTS {
                    sleep(IO_RETRY_DELAY);
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| std::io::Error::other("删除文件失败")))
}

/// SQLite 的 WAL / SHM 文件路径：在数据库文件名后直接追加后缀
fn sidecar_path(db_path: &Path, suffix: &str) -> PathBuf {
    let mut value = db_path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

fn remove_sidecars(db_path: &Path) {
    for suffix in ["-wal", "-shm"] {
        let _ = fs::remove_file(sidecar_path(db_path, suffix));
    }
}

/// 使用 `VACUUM INTO` 生成一致的数据库副本
///
/// 先写入同目录的临时文件并完整校验，再替换目标文件，避免备份过程中
/// 失败留下半截文件；目标已存在时按用户选择的「覆盖」处理。
fn backup_to(conn: &Connection, destination: &Path) -> Result<u64, String> {
    ensure_parent_directory(destination)?;

    let parent = match destination.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    let temp_path = parent.join(format!(".kgn-backup-{}.tmp", Uuid::new_v4()));
    let temp = temp_path
        .to_str()
        .ok_or_else(|| "目标路径包含无法识别的字符，请重新选择".to_string())?;

    if let Err(error) = conn.execute("VACUUM INTO ?1", [temp]) {
        let _ = fs::remove_file(&temp_path);
        return Err(format!("生成数据库备份失败：{error}"));
    }

    if let Err(error) = validate_database_file(&temp_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }

    if destination.exists()
        && let Err(error) = remove_file_with_retry(destination)
    {
        let _ = fs::remove_file(&temp_path);
        return Err(format!("覆盖已有备份文件失败：{error}"));
    }

    if let Err(error) = fs::rename(&temp_path, destination) {
        let _ = fs::remove_file(&temp_path);
        return Err(format!("保存备份文件失败：{error}"));
    }

    file_size(destination)
}

/// 在数据库同目录创建恢复前的安全副本
fn create_safety_backup(db_path: &Path) -> Result<PathBuf, String> {
    let parent = db_path
        .parent()
        .ok_or_else(|| "无法确定数据库所在目录".to_string())?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let mut candidate = parent.join(format!("knowledge_graph-before-restore-{stamp}.db"));
    let mut index = 1;

    while candidate.exists() {
        candidate = parent.join(format!("knowledge_graph-before-restore-{stamp}-{index}.db"));
        index += 1;
    }

    copy_with_retry(db_path, &candidate).map_err(|e| format!("创建恢复前备份失败：{e}"))?;

    Ok(candidate)
}

/// 用备份文件替换当前数据库
fn replace_database(source: &Path, db_path: &Path) -> Result<(), String> {
    let parent = db_path
        .parent()
        .ok_or_else(|| "无法确定数据库所在目录".to_string())?;
    let temp_path = parent.join(format!(".kgn-restore-{}.tmp", Uuid::new_v4()));

    copy_with_retry(source, &temp_path).map_err(|e| format!("复制备份文件失败：{e}"))?;

    if let Err(error) = validate_database_file(&temp_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }

    // 先清理日志文件，再替换主数据库，避免旧 WAL 与新数据库不匹配
    remove_sidecars(db_path);

    if db_path.exists()
        && let Err(error) = remove_file_with_retry(db_path)
    {
        let _ = fs::remove_file(&temp_path);
        return Err(format!("替换数据库失败：{error}"));
    }

    if let Err(rename_error) = fs::rename(&temp_path, db_path) {
        // 某些同步盘 / 杀毒软件会阻止重命名，退化为直接复制
        let copy_result = copy_with_retry(&temp_path, db_path);
        let _ = fs::remove_file(&temp_path);

        if let Err(copy_error) = copy_result {
            return Err(format!("写入数据库失败：{rename_error}；{copy_error}"));
        }
    }

    Ok(())
}

/// 恢复数据库：校验备份、创建安全副本、替换数据库
///
/// 返回恢复前自动创建的安全副本路径；失败时尽量回滚到恢复前的数据。
fn restore_from(source: &Path, db_path: &Path) -> Result<PathBuf, String> {
    if same_path(source, db_path) {
        return Err("请选择备份文件，而不是当前正在使用的数据库".into());
    }

    validate_database_file(source)?;
    let safety_backup = create_safety_backup(db_path)?;

    if let Err(error) = replace_database(source, db_path) {
        let _ = fs::copy(&safety_backup, db_path);
        return Err(format!(
            "恢复数据库失败：{error}（已尝试回滚到恢复前的数据）"
        ));
    }

    Ok(safety_backup)
}

/// 当前数据库的路径、大小与各表数据量
#[tauri::command]
pub fn get_database_info(app: AppHandle) -> Result<DatabaseInfo, String> {
    let path = database_path(&app)?;
    collect_database_info(&path)
}

/// 把当前数据库备份到用户选择的位置
#[tauri::command]
pub fn backup_database(app: AppHandle, destination: String) -> Result<BackupResult, String> {
    let destination = destination.trim();
    if destination.is_empty() {
        return Err("请选择备份文件的保存位置".into());
    }

    let db_path = database_path(&app)?;
    let destination = ensure_database_extension(PathBuf::from(destination));

    if same_path(&db_path, &destination) {
        return Err("不能把备份保存为当前正在使用的数据库".into());
    }

    let conn = get_connection(&app)?;
    let size_bytes = backup_to(&conn, &destination)?;

    Ok(BackupResult {
        path: path_display(&destination),
        size_bytes,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// 从备份文件恢复数据库
///
/// 恢复成功后立即重放幂等迁移，让旧版本备份补齐后续新增的表；
/// 迁移失败时回滚到恢复前的安全副本。
#[tauri::command]
pub fn restore_database(app: AppHandle, source: String) -> Result<RestoreResult, String> {
    let source = source.trim();
    if source.is_empty() {
        return Err("请选择要恢复的备份文件".into());
    }

    let source = PathBuf::from(source);
    let db_path = database_path(&app)?;
    let safety_backup = restore_from(&source, &db_path)?;

    if let Err(error) = initialize_database(&app) {
        let _ = fs::copy(&safety_backup, &db_path);
        return Err(format!(
            "恢复后升级数据库结构失败：{error}（已回滚到恢复前的数据）"
        ));
    }

    Ok(RestoreResult {
        source_path: path_display(&source),
        safety_backup_path: path_display(&safety_backup),
        restored_at: chrono::Utc::now().to_rfc3339(),
        database: collect_database_info(&db_path)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("kgn-backup-{}", Uuid::new_v4()));
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

    fn create_database(path: &Path, note_count: i64) {
        let conn = Connection::open(path).expect("打开测试数据库失败");
        conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
            .expect("初始化数据库失败");
        conn.execute_batch(include_str!("../../migrations/002_relations.sql"))
            .expect("执行关联迁移失败");
        conn.execute_batch(
            "
            INSERT INTO courses (id, name, description, created_at, updated_at)
            VALUES ('c1', '数据结构', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');

            INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at)
            VALUES ('kp1', 'c1', NULL, '链表', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
            ",
        )
        .expect("写入测试课程失败");

        for index in 0..note_count {
            conn.execute(
                "INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at)
                 VALUES (?1, 'kp1', ?2, '正文', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                rusqlite::params![format!("n{index}"), format!("笔记 {index}")],
            )
            .expect("写入测试笔记失败");
        }
    }

    fn count_rows(path: &Path, table: &str) -> i64 {
        let conn = Connection::open(path).expect("打开数据库失败");
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .expect("统计数据失败")
    }

    #[test]
    fn backup_creates_valid_database_copy() {
        let dir = TempDir::new();
        let db_path = dir.join("knowledge_graph.db");
        create_database(&db_path, 2);

        let conn = Connection::open(&db_path).expect("打开数据库失败");
        let backup_path = dir.join("backup.db");
        let size = backup_to(&conn, &backup_path).expect("备份数据库失败");

        assert!(size > 0);
        assert!(backup_path.exists());
        validate_database_file(&backup_path).expect("备份文件应通过校验");
        assert_eq!(count_rows(&backup_path, "notes"), 2);
        assert_eq!(count_rows(&backup_path, "knowledge_points"), 1);
    }

    #[test]
    fn backup_overwrites_existing_target() {
        let dir = TempDir::new();
        let db_path = dir.join("knowledge_graph.db");
        create_database(&db_path, 1);

        let conn = Connection::open(&db_path).expect("打开数据库失败");
        let backup_path = dir.join("backup.db");
        fs::write(&backup_path, b"old backup").expect("写入旧文件失败");

        backup_to(&conn, &backup_path).expect("备份数据库失败");
        validate_database_file(&backup_path).expect("覆盖后的备份应有效");
        assert_eq!(count_rows(&backup_path, "notes"), 1);
    }

    #[test]
    fn restore_replaces_database_and_keeps_safety_copy() {
        let dir = TempDir::new();
        let db_path = dir.join("knowledge_graph.db");
        create_database(&db_path, 1);

        let conn = Connection::open(&db_path).expect("打开数据库失败");
        let backup_path = dir.join("backup.db");
        backup_to(&conn, &backup_path).expect("备份数据库失败");
        drop(conn);

        // 备份之后再写入一条笔记，恢复后应回到备份时的状态
        let conn = Connection::open(&db_path).expect("打开数据库失败");
        conn.execute(
            "INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at)
             VALUES ('n-extra', 'kp1', '新增笔记', '', '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z')",
            [],
        )
        .expect("写入额外笔记失败");
        drop(conn);
        assert_eq!(count_rows(&db_path, "notes"), 2);

        let safety = restore_from(&backup_path, &db_path).expect("恢复数据库失败");

        assert!(safety.exists());
        assert_eq!(count_rows(&db_path, "notes"), 1);
        assert_eq!(count_rows(&safety, "notes"), 2);
        validate_database_file(&db_path).expect("恢复后的数据库应有效");
    }

    #[test]
    fn restore_rejects_current_database_and_invalid_files() {
        let dir = TempDir::new();
        let db_path = dir.join("knowledge_graph.db");
        create_database(&db_path, 1);

        let current = restore_from(&db_path, &db_path);
        assert!(current.is_err());

        let text_path = dir.join("notes.txt");
        fs::write(&text_path, b"not a database").expect("写入文本文件失败");
        let invalid = restore_from(&text_path, &db_path);
        assert!(invalid.is_err());

        let missing_table = dir.join("partial.db");
        let conn = Connection::open(&missing_table).expect("创建数据库失败");
        conn.execute_batch("CREATE TABLE courses (id TEXT PRIMARY KEY);")
            .expect("创建表失败");
        drop(conn);
        let partial = restore_from(&missing_table, &db_path);
        assert!(partial.is_err());

        // 失败的恢复不会改动当前数据库
        assert_eq!(count_rows(&db_path, "notes"), 1);
    }

    #[test]
    fn database_info_reports_path_size_and_counts() {
        let dir = TempDir::new();
        let db_path = dir.join("knowledge_graph.db");
        create_database(&db_path, 3);

        let info = collect_database_info(&db_path).expect("读取数据库信息失败");
        assert_eq!(info.counts.course_count, 1);
        assert_eq!(info.counts.knowledge_point_count, 1);
        assert_eq!(info.counts.note_count, 3);
        assert!(info.size_bytes > 0);
        assert_eq!(
            Path::new(&info.path),
            db_path.as_path(),
            "应返回数据库绝对路径"
        );
    }
}
