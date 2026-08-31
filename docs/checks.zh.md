# 检查

> [English](checks.md) | 简体中文

快门在每次提交前运行,重门在每次推送前运行;CI 在每次推送 / PR 时通过
`just check` 执行整条链。

## 工具

检查门依赖四个外部工具;`just setup` 会自动安装缺失的(并激活 git hook):

```bash
cargo install cargo-machete cargo-audit cargo-outdated cargo-deny --locked
```

`cargo fmt` 和 `cargo clippy` 由 `rust-toolchain.toml` 声明的工具链自带。

## 每次提交 —— `githooks/pre-commit`

| # | 检查 | 命令 | 用途 |
|---|------|------|------|
| 1 | fmt | `cargo fmt --all -- --check` | 代码格式 |
| 2 | secrets | `githooks/check-secrets` | 暂存区密钥扫描 |
| 3 | machete | `cargo machete` | 未使用的依赖 |
| 4 | docs | `githooks/check-docs` | 文档与代码对齐 |
| 5 | clippy | `cargo clippy --all-targets --all-features -- -D warnings` | 严格 lint |

某一行必须出现密钥形态的字符串时(如密钥格式说明文档),在该行加
`security-scan:allow` 标记并注明原因;`check-secrets` 会跳过它。

## 每次推送 —— `githooks/pre-push`

| # | 检查 | 命令 | 用途 |
|---|------|------|------|
| 6 | audit | `cargo audit` | RustSec 安全通告 |
| 7 | deny | `cargo deny check` | 许可证 / 禁用项 / 通告策略 |
| 8 | outdated | `cargo outdated --root-deps-only` | 直接依赖是否过期 |
| 9 | test | `cargo test --quiet` | 测试套件 |

## 一键运行

```bash
just check   # 与 hook + CI 完全一致
```

## 检查门拦住了你怎么办

先修代码。放行是最后手段:只能代码级局部放松(优先 `#[expect(...)]` 而非
`#[allow]`)、最小作用域、附原因注释;禁止削弱 `[lints]`、hook 或 CI。
详见 [Lint 策略](lint-policy.zh.md) 与 [AGENTS.md](../AGENTS.md)。
