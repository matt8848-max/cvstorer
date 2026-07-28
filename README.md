# 简历管理系统 (Resume Manager)

基于 Rust 的个人简历条目与模板存储管理系统。

## 概述

本系统以渐进式 Markdown 文件存储个人信息（条目），条目使用 **YAML frontmatter + Markdown 正文** 格式。通过 CLI 管理条目和 LaTeX 模板，PDF 渲染由外部工具完成。

CLI 基于 tokio 异步运行时，使用 clap 提供结构化的、面向 AI 的帮助文档。所有数据均为纯文本文件，可版本控制、AI 可读。

## 项目结构

```
├── resume/                 ← Rust CLI 项目
│   ├── Cargo.toml
│   ├── config.toml         ← 全局配置
│   └── src/
│       ├── lib.rs          ← 统一模块导出
│       ├── main.rs         ← tokio::main, clap 解析
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
├── CLAUDE.md               ← AI 入口文档
├── README.md               ← 本文件
└── LICENSE
```

## CLI 功能

### 条目管理

| 命令 | 说明 | 示例 |
|------|------|------|
| `entry list [--filter <tags>]` | 列出条目，可按标签筛选 | `resume entry list --filter experience,education` |
| `entry add <name> <path>` | 从文件添加条目（命名应详细，便于 AI 理解） | `resume entry add my-experience ./resume.md` |
| `entry path <name>` | 获取条目文件绝对路径 | `resume entry path my-experience` |
| `entry remove <name>` | 删除条目 | `resume entry remove my-experience` |

### 模板管理

| 命令 | 说明 | 示例 |
|------|------|------|
| `template list` | 列出已安装模板（含描述） | `resume template list` |
| `template add <name> <path> <desc>` | 添加模板（描述为必填位置参数，至多 100 字 UTF-8） | `resume template add my-template ./template.tex "简约现代风格，适合 IT 行业"` |
| `template path <name>` | 获取模板文件绝对路径 | `resume template path my-template` |
| `template remove <name>` | 删除模板（自动清理描述） | `resume template remove my-template` |

## 数据格式

条目使用 YAML frontmatter + Markdown 正文格式：

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

模板描述信息存储于 `data/templates/_manifest.yaml`：

```yaml
descriptions:
  moderncv: "简约现代风格，适合 IT 行业简历"
  fancy: "带蓝色标题栏的华丽风格，适合创意行业"
```

## 技术栈

- **语言**: Rust（类型安全、零成本抽象）
- **CLI**: clap（结构化帮助文档，AI 友好）
- **存储**: 纯文本 YAML + Markdown（可版本控制，AI 可读）
- **运行时**: tokio（异步运行时）
- **日志**: log + env_logger（通过 `RUST_LOG=info` 或 `RUST_LOG=debug` 控制）
- **依赖**: clap, serde, serde_yaml, toml, tokio, anyhow, log, env_logger

## 快速开始

### 构建

```bash
cd resume
cargo build --release
```

### 初始化

首次运行时自动初始化数据存储目录。

```bash
# 查看帮助
./target/release/resume --help

# 列出条目
./target/release/resume entry list

# 添加模板
./target/release/resume template add moderncv ./path/to/template.tex "简约现代风格，适合 IT 行业"
```

### 日志控制

```bash
RUST_LOG=info ./target/release/resume entry list
RUST_LOG=debug ./target/release/resume entry list
```

## 许可证

本项目基于 MIT 许可证开源 — 详见 [LICENSE](./LICENSE) 文件。