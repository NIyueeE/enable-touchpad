# 发布

> [English](release.md) | 简体中文

发布只需推送一个标签:

```bash
git tag v0.1.0
git push origin v0.1.0
```

`.github/workflows/release.yml` 会先创建 GitHub Release,再为三个目标平台构建
二进制,并把压缩包挂到 Release 上:

| Runner | 目标 |
|--------|------|
| ubuntu-latest | x86_64-unknown-linux-gnu |
| macos-latest | aarch64-apple-darwin |
| windows-latest | x86_64-pc-windows-msvc |

每个压缩包内含二进制,以及 `LICENSE`、`LICENSE-MIT`、`LICENSE-APACHE` 和
`README.md`。

## 测试构建(不是发布)

`.github/workflows/test-build.yml` 可以为任意 commit 构建指定平台的**测试
产物**——在 Actions 页手动触发(**Test build → Run workflow**),选择 `ref`
(commit SHA、分支或标签)和 `targets`(`linux`、`macos`、`windows`)。产物
仅保留 7 天,绝不会作为 Release 发布。发布永远由标签驱动。

## 注意事项

- **发布说明来自 `CHANGELOG.md`**(Keep a Changelog 格式):开发中的改动先记在
  `## [Unreleased]` 小节,打标签前把它改名为对应版本小节。缺失该小节时工作流
  会直接失败。
- 推标签前,先把 `Cargo.toml` 的 `version` 与标签对齐(手动改,或用 release-plz
  之类的工具自动化)。
- 该工作流需要 `contents: write` 权限——文件里已声明。
- 构建失败重跑:删除并重推标签(`git push origin :v0.1.0`),或在 Actions 页面
  直接 re-run。
