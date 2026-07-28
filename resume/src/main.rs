//! 简历管理系统 — CLI 入口
//!
//! tokio 异步运行时入口，解析 CLI 参数并分发到各模块处理。
//! 所有命令的 `--help` 输出面向 AI 设计。
//! 日志通过 `env_logger` 控制，设置 `RUST_LOG=info` 或 `RUST_LOG=debug` 开启。

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

use resume::cli::{Cli, Commands, EntryCommands, TemplateCommands};
use resume::entry;
use resume::store;
use resume::template;

#[tokio::main]
async fn main() {
    // 初始化日志系统，通过 RUST_LOG 环境变量控制日志级别
    env_logger::init();

    // 首次运行时自动初始化数据存储目录
    if let Err(e) = auto_init() {
        log::warn!("自动初始化失败: {:#}", e);
    }

    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Entry(cmd) => handle_entry(cmd),
        Commands::Template(cmd) => handle_template(cmd),
    };

    if let Err(e) = result {
        log::error!("{:#}", e);
        std::process::exit(1);
    }
}

/// 自动初始化数据存储目录和配置文件
///
/// 首次运行时自动创建 config.toml（如果不存在）以及所需的目录结构。
fn auto_init() -> Result<()> {
    let config = store::Config::load()?;

    // 自动创建存储根目录、条目目录、模板目录
    let dirs = [
        &config.store.path,
        &config.entries_dir(),
        &config.templates_dir(),
    ];

    for dir in dirs {
        if !dir.exists() {
            log::info!("自动创建目录: {}", dir.display());
            std::fs::create_dir_all(dir)
                .with_context(|| format!("无法创建目录: {}", dir.display()))?;
        }
    }

    Ok(())
}

/// 处理 `entry` 子命令
fn handle_entry(cmd: &EntryCommands) -> Result<()> {
    let config = store::Config::load()?;
    let entries_dir = config.entries_dir();

    match cmd {
        EntryCommands::List { filter } => {
            let files = store::list_entry_files(&entries_dir)?;
            if files.is_empty() {
                println!("暂无条目");
                return Ok(());
            }

            let filter_tags: Vec<&str> = filter
                .as_deref()
                .map(|f| f.split(',').map(|s| s.trim()).collect())
                .unwrap_or_default();

            log::info!("列出条目 (过滤标签: {:?})", filter_tags);

            for file in &files {
                let entry = entry::load_entry(file)?;

                if !filter_tags.is_empty() {
                    let has_tag = filter_tags
                        .iter()
                        .any(|tag| entry.meta.tags.contains(&tag.to_string()));
                    if !has_tag {
                        continue;
                    }
                }

                let name = entry::entry_name(file).unwrap_or_else(|| "?".to_string());
                let date_info = match (&entry.meta.date_start, &entry.meta.date_end) {
                    (Some(s), Some(e)) => format!("{} - {}", s, e),
                    (Some(s), None) => format!("{} - 至今", s),
                    (None, _) => String::new(),
                };
                let tags = entry.meta.tags.join(", ");
                println!(
                    "  📄 {}{}{}",
                    name,
                    if tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", tags)
                    },
                    if date_info.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", date_info)
                    },
                );
            }
            Ok(())
        }

        EntryCommands::Add { name, path } => {
            log::info!("从文件添加条目: {} (名称: {})", path, name);
            let source = PathBuf::from(path);
            let entry = entry::create_entry_from_file(&source, name, &entries_dir)?;
            entry::save_entry(&entry)?;
            println!("✅ 条目已添加: {}", entry.file_path.display());
            log::info!("条目已保存: {}", entry.file_path.display());
            Ok(())
        }

        EntryCommands::Path { name } => {
            let path = entry::entry_path(name, &entries_dir)?;
            println!("{}", path.display());
            Ok(())
        }

        EntryCommands::Remove { name } => {
            let file_path = entries_dir.join(format!("{}.md", name));
            if !file_path.exists() {
                anyhow::bail!("条目 '{}' 未找到", name);
            }
            entry::delete_entry(&file_path)?;
            log::info!("条目已删除: {}", name);
            println!("✅ 条目已删除: {}", name);
            Ok(())
        }
    }
}

/// 处理 `template` 子命令
fn handle_template(cmd: &TemplateCommands) -> Result<()> {
    let config = store::Config::load()?;
    let templates_dir = config.templates_dir();

    match cmd {
        TemplateCommands::List => {
            let templates = template::list_templates(&templates_dir)?;
            if templates.is_empty() {
                println!("暂无已安装的模板");
                return Ok(());
            }
            for t in &templates {
                if t.description.is_empty() {
                    println!("  📄 {}", t.name);
                } else {
                    println!("  📄 {} — {}", t.name, t.description);
                }
            }
            log::info!("列出 {} 个模板", templates.len());
            Ok(())
        }

        TemplateCommands::Add { name, path, desc } => {
            let source = PathBuf::from(path);
            let t = template::add_template(&source, name, desc, &templates_dir)?;
            log::info!("模板已添加: {} (来源: {})", t.name, source.display());
            println!("✅ 模板已添加: {} — {}", t.name, t.description);
            Ok(())
        }

        TemplateCommands::Path { name } => {
            let path = template::template_path(name, &templates_dir)?;
            println!("{}", path.display());
            Ok(())
        }

        TemplateCommands::Remove { name } => {
            template::remove_template(name, &templates_dir)?;
            log::info!("模板已删除: {}", name);
            println!("✅ 模板已删除: {}", name);
            Ok(())
        }
    }
}

