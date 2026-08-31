# rust-agents-template

> 一个 Rust 项目模板:严格 lint、分层 git hook、cargo-deny、CI/CD。

[![CI](https://github.com/NIyueeE/rust-agents-template/actions/workflows/ci.yml/badge.svg)](https://github.com/NIyueeE/rust-agents-template/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

[English](README.md) | [简体中文](README.zh.md)

一个开箱即用、带强约束的 Rust 二进制项目起点——严格的 lint 策略 + 分层自动化
检查(pre-commit / pre-push / CI)。

## 特性

- **最新 stable 工具链** —— `rust-toolchain.toml` 声明 `channel = "stable"`,
  任何人 clone 后 rustup 都会自动解析当时的最新 stable 版本,并自带
  `clippy` 与 `rustfmt` 组件。
- **严格 lint** —— `unsafe_code = "forbid"`,clippy `all` + `pedantic` 均为
  `deny`(见 [Lint 策略](docs/lint-policy.zh.md))。
- **分层检查门** —— 快门在每次提交前运行,重门在每次推送前运行,CI 强制同一套
  链(见 [检查](docs/checks.zh.md))。
- **一个标签即发布** —— 打 `v*` 标签自动构建多平台二进制(见
  [发布](docs/release.zh.md))。
- **Rust 2024 edition**。

## 快速开始

```bash
git clone https://github.com/NIyueeE/rust-agents-template.git
cd rust-agents-template

# 每个 clone 一次:激活 hook + 安装缺失工具
just setup   # (或手动:git config core.hooksPath githooks)

cargo run

# 随时手动跑整条检查链 —— 与 hook + CI 完全一致
just check
```

## 文档

| 文档 | 内容 |
|------|------|
| [docs/using-this-template.zh.md](docs/using-this-template.zh.md) | 从本模板派生新项目:改名清单 |
| [docs/checks.zh.md](docs/checks.zh.md) | 八道检查门、分层 hook、CI |
| [docs/lint-policy.zh.md](docs/lint-policy.zh.md) | 每条 lint 与级别、放行规则 |
| [docs/release.zh.md](docs/release.zh.md) | 打标签 → 多平台二进制发布 |
| [docs/structure.zh.md](docs/structure.zh.md) | 仓库里每个文件的用途 |
| [HANDOFF.md](HANDOFF.md) | 交接文档:当前工作状态、决策与开放事项 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 如何参与贡献 |
| [SECURITY.md](SECURITY.md) | 漏洞报告 |
| [AGENTS.md](AGENTS.md) | AI 编码代理(以及人类)的守则 |

每篇文档都有对应的 English 版本(同目录下去掉 `.zh` 后缀)。

## 参与贡献

欢迎 PR——参见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

基于 MIT OR Apache-2.0 许可证分发,详情见 [`LICENSE`](LICENSE)。

© 2026 NIyueeE(100502009+NIyueeE@users.noreply.github.com)
