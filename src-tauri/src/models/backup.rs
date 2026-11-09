use serde::Serialize;

use crate::db::TableCounts;

/// 当前数据库文件与数据量信息（设置页「数据与备份」展示）
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseInfo {
    /// 数据库文件的绝对路径
    pub path: String,
    /// 数据库所在目录（用于「打开所在文件夹」）
    pub directory: String,
    pub size_bytes: u64,
    /// 文件最后修改时间（ISO 8601）
    pub modified_at: Option<String>,
    pub sqlite_version: String,
    /// 各业务表记录数，展开到与其它字段同级
    #[serde(flatten)]
    pub counts: TableCounts,
}

/// 备份结果
#[derive(Debug, Clone, Serialize)]
pub struct BackupResult {
    /// 实际写入的备份文件路径
    pub path: String,
    pub size_bytes: u64,
    pub created_at: String,
}

/// 恢复结果
#[derive(Debug, Clone, Serialize)]
pub struct RestoreResult {
    /// 被恢复的备份文件路径
    pub source_path: String,
    /// 恢复前自动保存的安全副本路径
    pub safety_backup_path: String,
    pub restored_at: String,
    /// 恢复后重新读取的数据库信息
    pub database: DatabaseInfo,
}
