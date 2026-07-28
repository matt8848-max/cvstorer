//! 模板管理
//!
//! 管理 LaTeX 简历模板的添加、列表查看、删除和描述查询。
//! 模板以 `.tex` 文件形式存储在 templates 目录中，
//! 描述信息存储在 `_manifest.yaml` 文件中（供 AI 理解模板用途）。
//!
//! 描述至多 100 字（UTF-8 字符数）。

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 模板简易描述的最大长度（UTF-8 字符数）
pub const DESC_MAX_CHARS: usize = 100;

/// manifest 结构
#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    descriptions: BTreeMap<String, String>,
}

/// 模板信息
#[derive(Debug)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub file_path: PathBuf,
}

/// 获取 manifest 文件路径
fn manifest_path(templates_dir: &Path) -> PathBuf {
    templates_dir.join("_manifest.yaml")
}

/// 加载 manifest 文件
fn load_manifest(templates_dir: &Path) -> Result<Manifest> {
    let path = manifest_path(templates_dir);
    if !path.exists() {
        return Ok(Manifest {
            descriptions: BTreeMap::new(),
        });
    }
    let content = crate::store::fs::read_file(&path)
        .with_context(|| "读取模板 manifest 失败")?;
    let manifest: Manifest = serde_yaml::from_str(&content)
        .with_context(|| "解析模板 manifest 失败")?;
    Ok(manifest)
}

/// 保存 manifest 文件
fn save_manifest(templates_dir: &Path, manifest: &Manifest) -> Result<()> {
    let content = serde_yaml::to_string(manifest)
        .with_context(|| "序列化模板 manifest 失败")?;
    let path = manifest_path(templates_dir);
    crate::store::fs::write_file(&path, &content)
}

/// 列出已安装的模板（含描述）
pub fn list_templates(templates_dir: &Path) -> Result<Vec<Template>> {
    let mut templates = Vec::new();
    if !templates_dir.exists() {
        log::debug!("模板目录不存在: {}", templates_dir.display());
        return Ok(templates);
    }

    let manifest = load_manifest(templates_dir)?;

    for entry in std::fs::read_dir(templates_dir).context("读取模板目录失败")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("tex") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                let description = manifest
                    .descriptions
                    .get(name)
                    .cloned()
                    .unwrap_or_default();
                templates.push(Template {
                    name: name.to_string(),
                    description,
                    file_path: path,
                });
            }
        }
    }

    templates.sort_by(|a, b| a.name.cmp(&b.name));
    log::debug!("模板列表: 找到 {} 个模板", templates.len());
    Ok(templates)
}

/// 添加模板：将 `.tex` 文件复制到模板目录，并记录描述
///
/// `desc` 必须非空且不超过 `DESC_MAX_CHARS` 字符（UTF-8）。
pub fn add_template(
    source: &Path,
    name: &str,
    desc: &str,
    templates_dir: &Path,
) -> Result<Template> {
    // 校验描述长度
    let desc = desc.trim();
    if desc.is_empty() {
        return Err(anyhow!("模板描述不能为空"));
    }
    if desc.chars().count() > DESC_MAX_CHARS {
        return Err(anyhow!(
            "模板描述过长：{} 字，最多允许 {} 字（UTF-8）",
            desc.chars().count(),
            DESC_MAX_CHARS,
        ));
    }

    let dest = templates_dir.join(format!("{}.tex", name));

    // 确保模板目录存在再复制
    std::fs::create_dir_all(templates_dir)
        .with_context(|| format!("无法创建模板目录: {}", templates_dir.display()))?;

    std::fs::copy(source, &dest)
        .with_context(|| format!("无法复制模板: {} -> {}", source.display(), dest.display()))?;

    // 写入描述到 manifest
    let mut manifest = load_manifest(templates_dir)?;
    manifest.descriptions.insert(name.to_string(), desc.to_string());
    save_manifest(templates_dir, &manifest)?;

    log::info!("模板已添加: {} (来源: {})", name, source.display());
    Ok(Template {
        name: name.to_string(),
        description: desc.to_string(),
        file_path: dest,
    })
}

/// 删除模板：删除模板文件及描述
pub fn remove_template(name: &str, templates_dir: &Path) -> Result<()> {
    let path = templates_dir.join(format!("{}.tex", name));
    if !path.exists() {
        return Err(anyhow!("模板 '{}' 未找到", name));
    }

    // 从 manifest 删除描述
    let mut manifest = load_manifest(templates_dir)?;
    manifest.descriptions.remove(name);
    save_manifest(templates_dir, &manifest)?;

    log::info!("删除模板: {}", name);
    crate::store::fs::delete_file(&path)
}

/// 查询模板描述
pub fn describe_template(name: &str, templates_dir: &Path) -> Result<Option<String>> {
    let manifest = load_manifest(templates_dir)?;
    Ok(manifest.descriptions.get(name).cloned())
}

/// 获取模板文件的绝对路径
///
/// 根据模板名称查找对应的 .tex 文件，存在则返回规范化后的绝对路径。
pub fn template_path(name: &str, templates_dir: &Path) -> Result<PathBuf> {
    let file_path = templates_dir.join(format!("{}.tex", name));
    if !file_path.exists() {
        return Err(anyhow!("模板 '{}' 未找到", name));
    }
    Ok(file_path.canonicalize()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// 测试：不存在的模板目录应返回空列表
    #[test]
    fn test_list_templates_should_return_empty_for_nonexistent_dir() {
        let dir = PathBuf::from("/tmp/nonexistent_templates_12345");
        let templates = list_templates(&dir).unwrap();
        assert!(templates.is_empty());
    }

    /// 测试：添加模板应复制文件并记录描述，删除应清理文件与描述
    #[test]
    fn test_add_and_remove_template_with_description() {
        let temp_dir = std::env::temp_dir().join("resume_test_templates_desc");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // 创建测试模板文件
        let source = temp_dir.join("test.tex");
        fs::write(&source, r"\documentclass{article}").unwrap();

        // 添加模板（含描述）
        let templates_dir = temp_dir.join("installed");
        let t = add_template(&source, "mytpl", "简约风格，适合 IT 简历", &templates_dir).unwrap();
        assert_eq!(t.name, "mytpl");
        assert_eq!(t.description, "简约风格，适合 IT 简历");
        assert!(t.file_path.exists());

        // 验证 manifest 存在
        let manifest_path = templates_dir.join("_manifest.yaml");
        assert!(manifest_path.exists());

        // 列出模板应含描述
        let list = list_templates(&templates_dir).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].description, "简约风格，适合 IT 简历");

        // 查询描述
        let desc = describe_template("mytpl", &templates_dir).unwrap();
        assert_eq!(desc, Some("简约风格，适合 IT 简历".to_string()));

        // 删除模板
        remove_template("mytpl", &templates_dir).unwrap();
        assert!(!t.file_path.exists());

        // 确认描述也被清理
        let desc = describe_template("mytpl", &templates_dir).unwrap();
        assert_eq!(desc, None);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// 测试：空描述应报错
    #[test]
    fn test_add_template_should_fail_on_empty_desc() {
        let temp_dir = std::env::temp_dir().join("resume_test_empty_desc");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let source = temp_dir.join("test.tex");
        fs::write(&source, r"\documentclass{article}").unwrap();

        let result = add_template(&source, "tpl", "  ", &temp_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不能为空"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// 测试：template_path 应返回规范化后的绝对路径
    #[test]
    fn test_template_path_should_return_canonicalized_path() {
        let temp_dir = std::env::temp_dir().join("resume_test_template_path");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // 创建测试模板文件
        fs::write(temp_dir.join("my-template.tex"), r"\documentclass{article}").unwrap();

        let result = template_path("my-template", &temp_dir).unwrap();
        assert!(result.is_absolute());
        assert!(result.ends_with("my-template.tex"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// 测试：不存在的模板应报错
    #[test]
    fn test_template_path_should_fail_for_nonexistent() {
        let temp_dir = std::env::temp_dir().join("resume_test_template_path_nonexist");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let result = template_path("nonexistent", &temp_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未找到"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// 测试：超长描述应报错
    #[test]
    fn test_add_template_should_fail_on_long_desc() {
        let temp_dir = std::env::temp_dir().join("resume_test_long_desc");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let source = temp_dir.join("test.tex");
        fs::write(&source, r"\documentclass{article}").unwrap();

        // 101 个汉字
        let long_desc = "测".repeat(101);
        let result = add_template(&source, "tpl", &long_desc, &temp_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("过长"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}