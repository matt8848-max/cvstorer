//! 公用的文件系统操作
//!
//! 提供文件读写、目录创建、删除等基础操作。所有模块通过 `store::fs` 调用，
//! 避免各模块自行实现重复的文件操作代码。

use anyhow::{Context, Result};
use std::path::Path;

/// 读取文件内容为字符串
///
/// # 参数
/// - `path`: 目标文件路径
///
/// # 返回值
/// 成功时返回文件内容的字符串
pub fn read_file(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("无法读取文件: {}", path.display()))?;
    log::debug!("文件读取成功: {} ({} 字节)", path.display(), content.len());
    Ok(content)
}

/// 写入字符串到文件
///
/// 自动创建目标文件所在目录（如果不存在）。
///
/// # 参数
/// - `path`: 目标文件路径
/// - `content`: 要写入的字符串内容
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    // 自动创建父目录，避免文件写入失败
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("无法创建目录: {}", parent.display()))?;
    }
    std::fs::write(path, content)
        .with_context(|| format!("无法写入文件: {}", path.display()))?;
    log::debug!("文件写入成功: {} ({} 字节)", path.display(), content.len());
    Ok(())
}

/// 删除文件
///
/// # 参数
/// - `path`: 要删除的文件路径
pub fn delete_file(path: &Path) -> Result<()> {
    std::fs::remove_file(path)
        .with_context(|| format!("无法删除文件: {}", path.display()))?;
    log::debug!("文件已删除: {}", path.display());
    Ok(())
}