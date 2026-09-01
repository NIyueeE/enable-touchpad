# enable-touchpad

> 按住层键即可临时找回触摸板:按住时触摸板恢复,松开后再次软关闭。

[![CI](https://github.com/NIyueeE/enable-touchpad/actions/workflows/ci.yml/badge.svg)](https://github.com/NIyueeE/enable-touchpad/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

[English](README.md) | [简体中文](README.zh.md)

> **平台:Windows 11(已适配)。** 需要精确式触摸板;其他平台已在
> 多平台适配层中规划。

一个 Windows 托盘小程序,把 kanata 键盘引擎作为库内嵌运行。按住可配置的层键
(默认 CapsLock)时激活 `mouse` 层,触摸板的软开/软关由系统完成;本应用
从不直接禁用设备。

## 特性

- **Windows 11 已适配** —— 精确式触摸板状态查询与软切换路径已在
  Windows 11 上验证(见 [demo/README.zh.md](demo/README.zh.md))。
- **内嵌 kanata 引擎** —— 无内核驱动、无独立进程;kanata v1.11 作为库
  跑在单个可执行文件里(LL-hook 捕获 + SendInput 输出)。
- **按键捕获而非下拉框** —— 点击一行,按下任意支持的键:字母、数字、
  F 键、修饰键、小键盘等都可绑定。`Escape` 取消,`CapsLock` 固定为层保持键。
- **状态看门狗** —— 层键未按住时周期采样官方精确式触摸板状态,并用同一
  组合键软修正漂移;主开关关闭后,触摸板交还给系统。
- **分层架构** —— `etp-core`(领域层)、`etp-platform`(唯一的多平台
  适配层)、`etp-ffi`(仅 Windows 的 FFI 叶子),应用层依赖 `Platform`
  trait 编写。
- **严格检查链** —— `rust-toolchain.toml` 声明 `channel = "stable"`,
  自带 `clippy` 与 `rustfmt`;`unsafe_code = "forbid"`,clippy `all` +
  `pedantic` 均为 `deny`(见 [Lint 策略](docs/lint-policy.zh.md));每次提交
  前跑快门,每次推送前跑重门,CI 强制同一套链(见
  [检查](docs/checks.zh.md))。
- **一个标签即发布** —— 打 `v*` 标签自动构建多平台二进制(见
  [发布](docs/release.zh.md))。
- **Rust 2024 edition**。

## 快速开始

```bash
git clone https://github.com/NIyueeE/enable-touchpad.git
cd enable-touchpad

# 开发:每个 clone 一次 —— 激活 hook + 安装缺失工具
just setup   # (或手动:git config core.hooksPath githooks)

cargo run    # Linux/macOS 打印占位说明;真实应用请在 Windows 上运行

# 随时手动跑整条检查链 —— 与 hook + CI 完全一致
just check
```

在 Windows 11 上,构建或下载 exe 并以管理员运行:按住层键启用触摸板,松开
后再次软关闭。右键托盘图标打开设置窗口;保存后会重新生成 kanata 配置并
热应用。用法与限制见 [demo/README.zh.md](demo/README.zh.md)。

## 文档

| 文档 | 内容 |
|------|------|
| [demo/README.zh.md](demo/README.zh.md) | Windows 11 用法、安装与 demo 限制 |
| [docs/using-this-template.zh.md](docs/using-this-template.zh.md) | 从本项目 fork 后改名:改名清单 |
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
