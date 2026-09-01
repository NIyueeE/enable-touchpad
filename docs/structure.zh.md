# 项目结构

> [English](structure.md) | 简体中文

| 路径 | 用途 |
|------|------|
| `src/main.rs` | 二进制入口 |
| `src/bin/enable-touchpad/` | Windows demo 二进制:托盘 + 设置页 + 内嵌 kanata + 触摸板状态看门狗(其他目标为占位桩) |
| `etp-ffi/` | 精确式触摸板状态查询的最小 FFI crate(隔离 `unsafe`)|
| `demo/` | Windows 可行性 demo 说明:双语 README |
| `tests/cli.rs` | 二进制冒烟测试 |
| `Cargo.toml` | 清单:严格 `[lints]`、包元数据 |
| `Cargo.lock` | 入库提交(二进制模板惯例) |
| `rust-toolchain.toml` | `channel = "stable"` + clippy/rustfmt 组件 |
| `justfile` | `just setup`(hook + 工具)/ `just check`(全链) |
| `deny.toml` | cargo-deny 策略:许可证 / 禁用项 / 通告 / 来源 |
| `githooks/pre-commit` | 快门:fmt、secrets、machete、docs、clippy |
| `githooks/pre-push` | 重门:audit、deny、outdated、test |
| `githooks/check-docs` | 文档与代码对齐门 |
| `githooks/check-secrets` | 暂存区密钥扫描 |
| `.github/workflows/ci.yml` | CI:推送 / PR 时运行 `just check` |
| `.github/workflows/release.yml` | 打 `v*` 标签 → 多平台二进制发布 |
| `.github/workflows/test-build.yml` | 手动为任意 commit 构建指定平台测试产物 |
| `.github/dependabot.yml` | 每周自动升级 actions + cargo 依赖 |
| `AGENTS.md` | AI 编码代理(以及人类)的守则 |
| `CONTRIBUTING.md` | 贡献指南 |
| `SECURITY.md` | 漏洞报告政策 |
| `LICENSE`(+ `LICENSE-MIT` / `LICENSE-APACHE`)| MIT OR Apache-2.0 |
| `.editorconfig` | 跨编辑器基础格式约定 |
| `docs/` | 模块化文档(本目录) |

延伸阅读:[检查](checks.zh.md) · [Lint 策略](lint-policy.zh.md) · [发布](release.zh.md)
