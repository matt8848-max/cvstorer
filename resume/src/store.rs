//! 配置管理与目录初始化
//!
//! 管理全局配置的加载、存储目录的初始化。
//! 公用的文件读写操作委托给 [`fs`] 子模块，其他模块通过 `store::fs` 调用。

pub mod fs;

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// 全局配置
#[derive(Debug, Deserialize)]
pub struct Config {
    pub store: StoreConfig,
}

/// 存储配置
#[derive(Debug, Deserialize)]
pub struct StoreConfig {
    pub path: PathBuf,
}

impl Config {
    /// 从默认路径加载配置
    ///
    /// 如果配置文件不存在，自动用编译时嵌入的默认内容创建。
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if !config_path.exists() {
            log::info!("配置文件不存在，创建默认配置: {}", config_path.display());
            let default_content = include_str!("../config.toml");
            fs::write_file(&config_path, default_content)?;
            log::info!("默认配置文件已创建");
        }

        log::debug!("加载配置文件: {}", config_path.display());
        let content = fs::read_file(&config_path)?;
        let config: Config = toml::from_str(&content)
            .with_context(|| "配置文件格式无效")?;
        log::info!("配置已加载 (存储路径: {})", config.store.path.display());
        Ok(config)
    }

    /// 获取配置文件的默认路径（运行目录下的 config.toml）
    fn config_path() -> PathBuf {
        PathBuf::from("config.toml")
    }

    /// 获取条目存储目录
    pub fn entries_dir(&self) -> PathBuf {
        self.store.path.join("entries")
    }

    /// 获取模板存储目录
    pub fn templates_dir(&self) -> PathBuf {
        self.store.path.join("templates")
    }
}

/// 列出目录下的所有条目文件（.md / .yaml）
pub fn list_entry_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    if !dir.exists() {
        log::debug!("条目目录不存在: {}", dir.display());
        return Ok(entries);
    }

    for entry in std::fs::read_dir(dir).context("读取条目目录失败")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            match path.extension().and_then(|e| e.to_str()) {
                Some("md" | "yaml" | "yml") => entries.push(path),
                _ => {}
            }
        }
    }

    entries.sort();
    log::debug!("条目目录扫描完成: 找到 {} 个文件", entries.len());
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow;

    /// 测试：空字符串反序列化应失败，验证 TOML 解析器正确工作
    #[test]
    fn test_toml_parse_should_fail_on_empty_string() {
        let result: Result<Config> = toml::from_str("").map_err(|e| anyhow::anyhow!(e));
        assert!(result.is_err());
    }

    /// 测试：不存在的条目目录应返回空列表，而不是报错
    #[test]
    fn test_list_entry_files_should_return_empty_for_nonexistent_dir() {
        let dir = PathBuf::from("/tmp/nonexistent_dir_12345");
        let result = list_entry_files(&dir).unwrap();
        assert!(result.is_empty());
    }
}