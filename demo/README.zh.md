# Windows 可行性 Demo

> English | [README.md](README.md)

`enable-touchpad` 设想在 Windows 上的概念验证,交付形态是**单个可执行文件**:

- **kanata 以库的形式内嵌在本应用里**(kanata v1.11,低级钩子捕获——无需内核
  驱动、无独立进程)。按住 `CapsLock` 激活 `mouse` 层(`Q`/`W`/`E` → 鼠标按键,
  `Left Alt` → `CapsLock`),并且 **Ctrl+Win+F24 在按下时、松开时各发出一次**。
  触摸板的软开/软关由系统(触摸板驱动对该组合键的绑定)执行——本应用自身
  不启用也不禁用任何设备。
- **Dioxus UI** 是一个小的设置窗口(mouse 层按键绑定配置),默认隐藏,从托盘
  打开;改动通过内嵌 kanata 的本地 TCP `Reload` 命令即时热生效。

## 目录结构

| 路径 | 用途 |
|------|------|
| `../src/bin/enable-touchpad/` | 应用代码:`main` / `app`(UI + 日志)/ `config` / `kanata_embed`(配置生成 + 内嵌 kanata)/ `tray` |

应用只在 Windows 下编译;其他目标编译为占位桩,保证仓库的 Linux 门禁不受影响。

## Windows 部署步骤

1. **构建或下载** exe(Windows 上 `cargo build --release`,或取 CI 的
   test-build 产物)。
2. **以管理员身份运行**。生成的 kanata 配置位于
   `%APPDATA%\enable-touchpad\kanata.kbd`;日志位于
   `%APPDATA%\enable-touchpad\enable-touchpad.log`。
3. 按住 `CapsLock` → `mouse` 层激活(按键绑定可配置),系统执行触摸板软开关。
4. 设置:右键托盘图标 → `打开设置`。保存后自动重新生成配置并热重载内嵌
   kanata——无需重启。

**不要同时再运行一个外部 kanata**(会造成双重按键捕获)。

## Demo 局限

- Ctrl+Win+F24 的"按下=开启、松开=关闭"语义取决于系统触摸板驱动如何处理该
  组合键;若驱动行为不同,调整 `%APPDATA%\enable-touchpad\kanata.kbd` 即可
  (它是唯一的配置落点)。
- 触摸板启停完全交给系统——组合键未绑定的机器上应用没有兜底手段。
- UI 渲染在系统 WebView 中(dioxus desktop);纯 GPU 原生渲染(Blitz)尚未成熟。
