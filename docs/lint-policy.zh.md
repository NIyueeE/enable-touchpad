# Lint 策略

> [English](lint-policy.md) | 简体中文

声明在 `Cargo.toml` 的 `[lints]` 表中:

| Lint | 级别 |
|------|------|
| `unsafe_code` | forbid |
| `missing_docs` | warn |
| clippy `all` | deny |
| clippy `pedantic` | deny |
| clippy `unwrap_used` / `expect_used` / `dbg_macro` | deny |
| clippy `todo` | warn |

pre-commit hook 额外传入 `-D warnings`,因此上述所有警告(包括 `missing_docs`
和 `todo`)在提交时都会升级为硬错误。

## 放行纪律

**修代码优先;放行是最后手段,且只能代码级局部放行。**

- 禁止通过改 `[lints]`、hook 或 CI 来"让错误消失"。
- 允许的放松:优先 `#[expect(clippy::lint_name)]`(lint 不再触发时会产生编译
  警告,避免遗留失效 allow),其次 `#[allow(clippy::lint_name)]`;仅限单条
  语句或单个函数级,并附一行原因注释。
- 禁止模块级 `#![allow(...)]` 或 crate 级放松。
- 仅有的两类合法场景:(1)业务上确实迫不得已;(2)上游误报(宏、生成代码、
  依赖自身的审计噪音)。

其他所有检查门(machete、audit、deny、outdated、docs-sync 以及未来新增的)
适用同一纪律。完整守则见 [AGENTS.md](../AGENTS.md)。
