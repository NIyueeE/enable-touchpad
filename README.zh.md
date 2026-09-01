# enable-touchpad

> 按住一个键,触摸板临时恢复;松开后,触摸板再次软关闭。

[![CI](https://github.com/NIyueeE/enable-touchpad/actions/workflows/ci.yml/badge.svg)](https://github.com/NIyueeE/enable-touchpad/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

[English](README.md) | [简体中文](README.zh.md)

`enable-touchpad` 是一个 Windows 托盘小工具,适合平时关闭触摸板、但又偶尔
需要短暂使用它的人。

按住层键(默认 CapsLock)时,触摸板可用;松开后,触摸板回到关闭状态。它
就像一个“触摸板临时开关”,不需要每次都进系统设置里来回切换。

> **平台:Windows 11(已适配)。** 需要精确式触摸板;其他平台尚未支持。

## 它解决什么问题

很多人会关闭触摸板,避免打字或使用鼠标时误触。但真正需要触摸板时,又得
打开 Windows 设置去找开关。

这个工具把这件事变成一个按键动作:

- 按住层键 → 触摸板开启
- 松开层键 → 触摸板再次关闭
- 退出程序 → 触摸板交还给系统

## 工作方式

程序监听一个配置好的层键。按住时启用鼠标层;松开时再次发送软关闭动作。
它还会检查官方的精确式触摸板状态,并在层键未按住时纠正状态漂移。

它**不会**卸载或禁用设备本身,只在运行期间做软切换。

## 使用方法

1. 下载或构建 Windows 可执行文件。
2. **以管理员身份运行**。
3. 在托盘找到程序图标。
4. 按住配置的层键(默认 CapsLock)即可启用触摸板。
5. 松开该键,触摸板会再次软关闭。
6. 右键托盘图标 → `打开设置`,可修改按键绑定或关闭主开关。

设置会即时热生效,不需要重启应用。

## 配置

- 可为鼠标左/中/右键以及 CapsLock 层动作绑定自己的按键。
- 按键通过直接按下捕获,而不是从下拉框里选。
- 主开关关闭后,触摸板完全交还给 Windows。
- 配置保存在 `%APPDATA%\enable-touchpad\`。
- 日志也保存在 `%APPDATA%\enable-touchpad\`。

## 要求与限制

- 需要 Windows 11 和精确式触摸板。
- 依赖系统/触摸板驱动对软切换行为的支持。
- 如果机器上软切换不可用,程序会记录日志,不会假装工作。
- 其他平台尚未支持。

## 项目说明

本项目使用 Rust 2024 edition 与稳定工具链
(`rust-toolchain.toml` 声明 `channel = "stable"`)。本地开发时,
用 `git config core.hooksPath githooks`(或 `just setup`)启用 hook,
用 `just check` 跑全链。项目文档:
`docs/using-this-template.zh.md`、`docs/checks.zh.md`、
`docs/lint-policy.zh.md`、`docs/release.zh.md`、`docs/structure.zh.md`
和 `HANDOFF.md`。

## 许可证

基于 MIT OR Apache-2.0 许可证分发,详情见 [`LICENSE`](LICENSE)。

© 2026 NIyueeE(100502009+NIyueeE@users.noreply.github.com)
