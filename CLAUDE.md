# 简历管理系统

## 项目概述

基于 Rust 的简历条目与模板存储管理系统。以渐进式 Markdown 文件存储个人信息，条目使用 YAML frontmatter + Markdown 正文格式，通过 CLI 管理条目和 LaTeX 模板。PDF 渲染由外部工具完成。

CLI 基于 tokio 异步运行时。首次运行时自动初始化数据存储目录。所有命令的 `--help` 输出面向 AI 设计。

## 项目结构

```
简历/
├── .agents/skills/resume/  ← Cline 技能（自动识别，按需加载）
├── resume/                 ← Rust CLI 项目
│   ├── Cargo.toml
│   ├── config.toml         ← 全局配置（仅 [store] 段）
│   └── src/
│       ├── lib.rs          ← 统一模块导出
│       ├── main.rs         ← tokio::main, clap parse
│       ├── cli.rs          ← 命令分发与 AI 友好 help
│       ├── entry.rs        ← 条目 CRUD
│       ├── template.rs     ← 模板 CRUD + _manifest.yaml 描述
│       ├── store.rs        ← 配置管理与目录初始化
│       └── store/
│           └── fs.rs       ← 公用的文件系统操作
├── config.toml             ← CLI 全局配置（运行目录）
├── data/
│   ├── entries/            ← 条目存储目录（.md 文件）
│   └── templates/          ← 模板存储目录（.tex + _manifest.yaml）
└── CLAUDE.md               ← 本文件 - AI 入口文档
```

## CLI 命令树

```
resume                                      # --help 输出完整 AI 指南
├── entry                                   # 条目管理
│   ├── list [--filter <tags>]             # 列出条目
│   │   └── 示例: resume entry list
│   │   └── 示例: resume entry list --filter experience,education
│   ├── add <name> <path>                  # 从文件添加条目（命名应详细，便于 AI 理解）
│   │   └── 示例: resume entry add my-experience ./resume.md
│   ├── path <name>                        # 获取条目文件绝对路径（AI 读取用）
│   │   └── 示例: resume entry path my-experience
│   └── remove <name>                      # 删除条目
│       └── 示例: resume entry remove my-experience
└── template                                # 模板管理
    ├── list                                # 列出已安装模板（含描述）
    │   └── 示例: resume template list
    ├── add <name> <path> <desc>           # 添加模板（描述为必填位置参数，至多 100 字 UTF-8）
    │   └── 示例: resume template add my-template ./template.tex "简约现代风格，适合 IT 行业"
    ├── path <name>                        # 获取模板文件绝对路径（AI 读取用）
    │   └── 示例: resume template path my-template
    └── remove <name>                      # 删除模板（自动清理描述）
        └── 示例: resume template remove my-template
```

## 数据格式规范

条目使用 **YAML frontmatter + Markdown 正文**：

```yaml
---
title: "条目标题"
tags: ["标签1", "标签2"]
date_start: "YYYY-MM"
date_end: "YYYY-MM"
category: experience
---
### 主要职责
- 职责描述

### 主要成就
- 成就描述
```

## 模板描述

模板的描述信息存储于 `data/templates/_manifest.yaml`，供 AI 理解模板用途：

```yaml
descriptions:
  moderncv: "简约现代风格，适合 IT 行业简历"
  fancy: "带蓝色标题栏的华丽风格，适合创意行业"
```

## 技术栈

- **语言**: Rust（类型安全、零成本抽象）
- **CLI**: clap（结构化帮助文档，AI 友好）
- **存储**: 纯文本 YAML+Markdown（可版本控制，AI 可读）
- **日志**: log + env_logger（通过 `RUST_LOG=info` 或 `RUST_LOG=debug` 控制）
- **依赖**: clap, serde, serde_yaml, toml, tokio, anyhow, log, env_logger