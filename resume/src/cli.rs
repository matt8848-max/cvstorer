//! CLI 命令定义与 AI 友好帮助文档
//!
//! 使用 clap 定义完整的命令树，每个子命令的 `--help` 输出面向 AI 设计，
//! 包含详细的参数说明和使用示例。
//!
//! 本 CLI 只负责条目存储和模板存储管理，不涉及 PDF 渲染。

use clap::{Parser, Subcommand};

/// 简历管理系统 — 条目与模板存储管理
///
/// 管理个人信息条目（YAML frontmatter + Markdown）和 LaTeX 模板。
/// PDF 渲染由外部工具完成。
///
/// # 快速开始
///
/// 1. `resume entry add my-entry ./resume.md` — 添加简历条目
/// 2. `resume entry list` — 列出所有条目
/// 3. `resume template add my-template ./template.tex "简约风格"` — 添加模板
///
/// # 数据格式
///
/// 条目使用 YAML frontmatter + Markdown 正文格式：
///
/// ```yaml
/// ---
/// title: "条目标题"
/// tags: ["标签"]
/// date_start: "2024-01"
/// date_end: "2024-12"
/// category: experience
/// ---
/// ### 职责
/// - 任务描述
/// ```
#[derive(Parser, Debug)]
#[command(name = "resume")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 条目管理
    #[command(subcommand)]
    Entry(EntryCommands),

    /// 模板管理
    #[command(subcommand)]
    Template(TemplateCommands),
}

#[derive(Subcommand, Debug)]
pub enum EntryCommands {
    /// 列出所有条目
    ///
    /// 可选的 --filter 参数按标签过滤（多个标签用逗号分隔）。
    ///
    /// # 示例
    /// `resume entry list`
    /// `resume entry list --filter experience,education`
    List {
        /// 按标签过滤（支持多个标签，用逗号分隔）
        #[arg(long)]
        filter: Option<String>,
    },

    /// 添加条目
    ///
    /// 条目存储为 YAML frontmatter + Markdown 格式。
    /// 建议使用详细命名（包含项目/技能/公司等关键词），便于 AI 理解。
    ///
    /// # 示例
    /// `resume entry add my-experience ./resume.md`
    ///
    /// 文件内容应为：
    /// ```yaml
    /// ---
    /// title: "条目标题"
    /// tags: ["标签"]
    /// ---
    /// ### 正文
    /// - 内容
    /// ```
    Add {
        /// 条目名称（存储为 `<name>.md`，后续通过此名称编辑/删除）。建议使用详细命名，便于 AI 理解。
        name: String,

        /// 源 Markdown 文件路径（含 YAML frontmatter）
        path: String,
    },

    /// 获取条目文件路径
    ///
    /// 返回条目文件的绝对路径，便于 AI 直接读取文件内容。
    ///
    /// # 示例
    /// `resume entry path my-experience`
    Path {
        /// 条目名称（不含扩展名）
        name: String,
    },

    /// 删除条目
    ///
    /// # 示例
    /// `resume entry remove my-experience`
    Remove {
        /// 条目名称（不含扩展名）
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TemplateCommands {
    /// 列出已安装的模板（含描述）
    ///
    /// # 示例
    /// `resume template list`
    List,

    /// 添加模板
    ///
    /// 模板为 .tex 文件，使用 xelatex 编译，必须支持 UTF-8 中文。
    /// 模板文件必须包含 {{CONTENT}} 占位符——待填充的条目内容会替换此占位符。
    /// 建议使用 xeCJK 或 ctex 宏包处理中文。
    ///
    /// 第三个参数为模板的简易描述（至多 100 字 UTF-8），供 AI 理解模板用途。
    ///
    /// # 示例
    /// `resume template add my-template ./template.tex "简约现代风格，适合 IT 行业"`
    Add {
        /// 模板名称（不含扩展名）
        name: String,

        /// 模板文件路径（.tex 文件）
        path: String,

        /// 模板描述，供 AI 理解模板用途。至多 100 字（UTF-8 字符数）。
        desc: String,
    },

    /// 获取模板文件路径
    ///
    /// 返回模板文件的绝对路径，便于 AI 直接读取文件内容。
    ///
    /// # 示例
    /// `resume template path my-template`
    Path {
        /// 模板名称（不含扩展名）
        name: String,
    },

    /// 删除模板（自动清理描述）
    ///
    /// # 示例
    /// `resume template remove my-template`
    Remove {
        /// 模板名称（不含扩展名）
        name: String,
    },
}
