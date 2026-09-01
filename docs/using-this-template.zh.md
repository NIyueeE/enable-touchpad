# 从本项目 fork 后改名

> [English](using-this-template.md) | 简体中文

本仓库最初是一个严格 Rust 模板,现在是 `enable-touchpad` 项目。如果你 fork
后要改名,按下面的清单走;改完可以用 grep 兜底确认:

```bash
grep -rn "enable-touchpad" . --exclude-dir={.git,target}
```

## 改名清单

| # | 文件 | 要改什么 |
|---|------|----------|
| 1 | `Cargo.toml` | `name`、`description`、`repository`;`version` 想重置就重置 |
| 2 | `Cargo.lock` | 无需手动改 —— `cargo check` 会自动再生(也可以先删掉) |
| 3 | `.github/workflows/release.yml` | `bin: enable-touchpad` → 新的二进制名 |
| 4 | `.github/workflows/test-build.yml` | artifact 路径 `enable-touchpad` / `enable-touchpad.exe` |
| 5 | `justfile` | 顶部注释(仅文案) |
| 6 | `README.md` / `README.zh.md` | 标题、徽章 URL、克隆 URL、简介文案 |
| 7 | `LICENSE` / `LICENSE-MIT` / `LICENSE-APACHE` | 版权持有人与年份 |
| 8 | `SECURITY.md`、`CONTRIBUTING.md`、`AGENTS.md` | 可选:调整联系方式 / 措辞 |
| 9 | `src/bin/enable-touchpad/` | 重命名二进制目录与 crate 级文档注释 |

**无需改动**的文件:`rust-toolchain.toml`、`deny.toml`、`githooks/*`、
`.editorconfig`、`docs/*`(均为相对链接)、`.github/dependabot.yml`。

## 改完之后

```bash
just setup        # 激活 hook + 安装工具
just check        # 全链检查 —— 会帮你抓出漏改的地方
git add -A && git commit -m "chore: rename project"   # pre-commit 在此自动运行
```

`just check` 是你的安全网:它会重新验证文档、hook、CI 与你刚改过的代码仍然
一致。

## 之后正常开发

- 提交跑快门,推送跑重门(见 [检查](checks.zh.md))
- 检查门拦住了你:先修代码 —— 放行只能代码级并留原因注释
  ([Lint 策略](lint-policy.zh.md))
- 要发布二进制:推一个 `v*` 标签([发布](release.zh.md))
