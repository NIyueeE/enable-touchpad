# 从本模板派生新项目

> [English](using-this-template.md) | 简体中文

点击 **Use this template** 之后,你的新仓库就已经自带严格 lint、分层 hook、
CI 和发布流水线了。接下来把项目改成*你的*名字。下面是需要改动的全部文件;
改完可以用 grep 兜底确认:

```bash
grep -rn "rust-agents-template" . --exclude-dir={.git,target}
```

## 改名清单

| # | 文件 | 要改什么 |
|---|------|----------|
| 1 | `Cargo.toml` | `name`、`description`、`repository`;`version` 想重置就重置 |
| 2 | `Cargo.lock` | 无需手动改 —— `cargo build` 会自动再生(也可以先删掉) |
| 3 | `tests/cli.rs` | `env!("CARGO_BIN_EXE_rust-agents-template")` → 新的二进制名 |
| 4 | `.github/workflows/release.yml` | `bin: rust-agents-template` → 新的二进制名 |
| 5 | `justfile` | 顶部注释(仅文案) |
| 6 | `README.md` / `README.zh.md` | 标题、徽章 URL、克隆 URL、简介文案 |
| 7 | `LICENSE` / `LICENSE-MIT` / `LICENSE-APACHE` | 版权持有人与年份 |
| 8 | `SECURITY.md`、`CONTRIBUTING.md`、`AGENTS.md` | 可选:调整联系方式 / 措辞 |
| 9 | `src/main.rs` | crate 级文档注释(missing_docs 强制) |

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
