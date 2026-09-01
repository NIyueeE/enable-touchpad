# Windows 可行性 Demo

> English | [README.md](README.md)

`enable-touchpad` 设想在 Windows 上的概念验证,交付形态是**单个可执行文件**:

- **kanata 以库的形式内嵌在本应用里**(kanata v1.11,低级钩子捕获——无需内核
  驱动、无独立进程)。按住 `CapsLock` 激活 `mouse` 层,并且 **Ctrl+Win+F24 在
  按下时、松开时各发出一次**。触摸板的软开/软关由系统(触摸板驱动对该组合键
  的绑定)执行——本应用自身不启用也不禁用任何设备。
- **层内按键通过"按下即捕获"配置**:点击设置页某行的按钮,再按下任意受支持的
  键(字母、数字、F 键、修饰键、小键盘……),鼠标左/中/右键与 CapsLock 动作
  各自绑定自己的键。`Esc` 取消捕获,`×` 把该行恢复为"无",`CapsLock` 本身
  固定为层触发键。
- **状态看门狗保证未按住层触发键时触摸板处于关闭**:每 ~1.2 秒读取官方
  精确式触摸板状态(`SPI_GETTOUCHPADPARAMETERS`,Win11+),发现漂移就用同一条
  Ctrl+Win+F24 软开关(经 kanata fake-key 通道)矫正。总开关关闭时触摸板
  归还给系统。
- **Dioxus UI** 是一个小的设置窗口,默认隐藏,从托盘打开;改动通过内嵌
  kanata 的本地 TCP `Reload` 命令即时热生效。

## 目录结构

| 路径 | 用途 |
|------|------|
| `../src/bin/enable-touchpad/` | 应用层:`main`(组合根)/ `app`(Dioxus UI)/ `config_store` / `logging` / `watchdog` / `tray` —— 全部基于 `Platform` trait 编写 |
| `../etp-core/` | 领域层:配置模型、键位白名单、kanata 配置生成器(跨平台,Linux 上单测)|
| `../etp-platform/` | 唯一的多平台适配层:`Platform` trait + `windows` 适配器(内嵌 kanata 引擎、层监控、触摸板状态)+ 非 Windows 兜底 |
| `../etp-ffi/` | 仅 Windows 的 FFI 叶子 crate:触摸板状态查询 `SPI_GETTOUCHPADPARAMETERS`(隔离 `unsafe`;应用 crate 禁用 unsafe)|

应用只在 Windows 下编译;其他目标编译为占位桩,保证仓库的 Linux 门禁不受影响。

## Windows 部署步骤

1. **构建或下载** exe(Windows 上 `cargo build --release`,或取 CI 的
   test-build 产物)。
2. **以管理员身份运行**。生成的 kanata 配置位于
   `%APPDATA%\enable-touchpad\kanata.kbd`;日志位于
   `%APPDATA%\enable-touchpad\enable-touchpad.log`。
3. 按住 `CapsLock` → `mouse` 层激活(按键绑定可捕获配置),系统执行触摸板软开关。
4. 设置:右键托盘图标 → `打开设置`。保存后自动重新生成配置并热重载内嵌
   kanata——无需重启。

**不要同时再运行一个外部 kanata**(会造成双重按键捕获)。

## Demo 局限

- Ctrl+Win+F24 的"按下=开启、松开=关闭"语义取决于系统触摸板驱动如何处理该
  组合键;若驱动行为不同,调整 `%APPDATA%\enable-touchpad\kanata.kbd` 即可
  (它是唯一的配置落点)。
- 触摸板启停完全交给系统——组合键未绑定的机器上应用没有兜底手段。状态看门狗
  依赖 Windows 11 + 精确式触摸板(`SPI_GETTOUCHPADPARAMETERS`);不满足时记录
  一次 "SPI unavailable" 后保持静默。
- 重复绑定按先选先得处理(一个键绑定到两个动作时只有第一个生效)。
- UI 渲染在系统 WebView 中(dioxus desktop);纯 GPU 原生渲染(Blitz)尚未成熟。
