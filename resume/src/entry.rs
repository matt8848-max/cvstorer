//! 条目 CRUD
//!
//! 管理简历条目的创建、读取、更新、删除操作。
//! 条目使用 YAML frontmatter + Markdown 正文格式存储。
//! 文件读写操作委托给 `store::fs` 子模块，避免重复实现。

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 条目标元数据（YAML frontmatter）
#[derive(Debug, Serialize, Deserialize)]
pub struct EntryMeta {
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub date_start: Option<String>,
    pub date_end: Option<String>,
    pub category: Option<String>,
}

/// 完整的条目（元数据 + 正文）
#[derive(Debug)]
pub struct Entry {
    pub meta: EntryMeta,
    pub body: String,
    pub file_path: PathBuf,
}

/// 解析包含 YAML frontmatter 的 Markdown 文件
///
/// 查找第一个 `---` 和第二个 `---` 之间的内容作为 YAML 元数据，剩余部分为正文。
fn parse_frontmatter(content: &str) -> Result<(EntryMeta, String)> {
    let content = content.trim();

    // 检查是否以 --- 开头，缺少则说明格式错误
    if !content.starts_with("---") {
        return Err(anyhow!("条目缺少 YAML frontmatter (---)"));
    }

    // 从第4个字符开始查找 \n---，找到结束标记的位置
    let end_marker = content[3..]
        .find("\n---")
        .ok_or_else(|| anyhow!("YAML frontmatter 未正确关闭 (---)"))?;

    // 提取 YAML 部分（不含 --- 定界符）和正文部分
    let yaml_str = &content[3..3 + end_marker].trim();
    let body = content[3 + end_marker + 4..].trim().to_string();

    let meta: EntryMeta = serde_yaml::from_str(yaml_str)
        .with_context(|| "解析 YAML frontmatter 失败")?;

    log::debug!("解析 frontmatter: title={}", meta.title);
    Ok((meta, body))
}

/// 将条目序列化为 YAML frontmatter + Markdown 格式
fn serialize_entry(meta: &EntryMeta, body: &str) -> Result<String> {
    let yaml_str = serde_yaml::to_string(meta)
        .with_context(|| "序列化 YAML frontmatter 失败")?;
    Ok(format!("---\n{}---\n{}", yaml_str, body))
}

/// 从文件加载条目
pub fn load_entry(path: &Path) -> Result<Entry> {
    log::debug!("加载条目文件: {}", path.display());
    let content = crate::store::fs::read_file(path)?;
    let (meta, body) = parse_frontmatter(&content)?;

    Ok(Entry {
        meta,
        body,
        file_path: path.to_path_buf(),
    })
}

/// 保存条目到文件
///
/// 将元数据序列化为 YAML frontmatter，与正文拼接后写入文件。
pub fn save_entry(entry: &Entry) -> Result<()> {
    log::debug!("保存条目: {}", entry.file_path.display());
    let content = serialize_entry(&entry.meta, &entry.body)?;
    crate::store::fs::write_file(&entry.file_path, &content)
}

/// 从文件路径创建条目
///
/// 读取指定文件的内容，解析后以指定名称存储到条目目录。
/// 名称由用户显式指定，而非从文件名提取。
pub fn create_entry_from_file(path: &Path, name: &str, entries_dir: &Path) -> Result<Entry> {
    log::debug!("从文件创建条目: {} (名称: {})", path.display(), name);
    let content = crate::store::fs::read_file(path)?;
    let (meta, body) = parse_frontmatter(&content)?;

    let file_path = entries_dir.join(format!("{}.md", name));

    Ok(Entry {
        meta,
        body,
        file_path,
    })
}

/// 删除条目
pub fn delete_entry(path: &Path) -> Result<()> {
    log::info!("删除条目文件: {}", path.display());
    crate::store::fs::delete_file(path)
}

/// 获取条目的显示名称（去掉扩展名）
pub fn entry_name(path: &Path) -> Option<String> {
    path.file_stem().and_then(|s| s.to_str()).map(String::from)
}

/// 获取条目文件的绝对路径
///
/// 根据条目名称查找对应的 .md 文件，存在则返回规范化后的绝对路径。
pub fn entry_path(name: &str, entries_dir: &Path) -> Result<PathBuf> {
    let file_path = entries_dir.join(format!("{}.md", name));
    if !file_path.exists() {
        return Err(anyhow!("条目 '{}' 未找到", name));
    }
    Ok(file_path.canonicalize()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试：标准 frontmatter + Markdown 正文应正确解析
    #[test]
    fn test_parse_frontmatter_should_handle_standard_input() {
        let input = "---\ntitle: \"测试条目\"\ntags: [\"工作\"]\n---\n### 职责\n- 任务描述";
        let (meta, body) = parse_frontmatter(input).unwrap();
        assert_eq!(meta.title, "测试条目");
        assert_eq!(meta.tags, vec!["工作"]);
        assert!(body.contains("### 职责"));
    }

    /// 测试：缺少 frontmatter 标记时应报错
    #[test]
    fn test_parse_frontmatter_should_fail_without_marker() {
        let input = "没有 frontmatter 的内容";
        assert!(parse_frontmatter(input).is_err());
    }

    /// 测试：只有 frontmatter 没有正文时应正确解析且正文为空
    #[test]
    fn test_parse_frontmatter_should_handle_empty_body() {
        let input = "---\ntitle: \"空正文\"\n---";
        let (meta, body) = parse_frontmatter(input).unwrap();
        assert_eq!(meta.title, "空正文");
        assert!(body.is_empty());
    }

    /// 测试：entry_path 应返回规范化后的绝对路径
    #[test]
    fn test_entry_path_should_return_canonicalized_path() {
        let temp_dir = std::env::temp_dir().join("resume_test_entry_path");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // 创建测试条目文件
        std::fs::write(
            temp_dir.join("test-entry.md"),
            "---\ntitle: \"测试\"\n---\n内容",
        )
        .unwrap();

        let result = entry_path("test-entry", &temp_dir).unwrap();
        assert!(result.is_absolute());
        assert!(result.ends_with("test-entry.md"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// 测试：不存在的条目应报错
    #[test]
    fn test_entry_path_should_fail_for_nonexistent() {
        let temp_dir = std::env::temp_dir().join("resume_test_entry_path_nonexist");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let result = entry_path("nonexistent", &temp_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未找到"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// 测试：序列化后再解析应保持数据一致（往返验证）
    #[test]
    fn test_serialize_and_parse_roundtrip() {
        let meta = EntryMeta {
            title: "测试".into(),
            tags: vec!["tag1".into()],
            date_start: Some("2024-01".into()),
            date_end: None,
            category: Some("experience".into()),
        };
        let body = "### 正文内容".to_string();

        let serialized = serialize_entry(&meta, &body).unwrap();
        let (parsed_meta, parsed_body) = parse_frontmatter(&serialized).unwrap();

        assert_eq!(parsed_meta.title, meta.title);
        assert_eq!(parsed_meta.tags, meta.tags);
        assert_eq!(parsed_meta.date_start, meta.date_start);
        assert_eq!(parsed_meta.date_end, meta.date_end);
        assert_eq!(parsed_meta.category, meta.category);
        assert_eq!(parsed_body, body);
    }
}