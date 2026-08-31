# Windows 可行性 Demo

> English | [README.md](README.md)

`enable-touchpad` 设想在 Windows 上的概念验证,交付形态是**单个可执行文件**:

- **kanata 以库的形式内嵌在本应用里**(kanata v1.11):通过低级键盘钩子捕获
  输入(**无需安装内核驱动**),按住 `CapsLock` 激活 `mouse` 层(`Q`/`W`/`E`
  → 鼠标左/中/右键,`Left Alt` → `CapsLock`),并把层状态通过进程内的本地
  TCP socket 广播。
- **Dioxus UI** 接收层信号,在**按住 CapsLock 期间启用触摸板、松开时禁用**,
  在鼠标位置显示可穿透点击的激活提示,并提供系统托盘图标和极简设置页。

```
enable-touchpad.exe(单文件)
├── kanata(内嵌库,LL 钩子捕获 + SendInput 注出)
│     ├── 按住 CapsLock → "mouse" 层 + Q/W/E = 鼠标按键
│     └── LayerChange → 127.0.0.1:<port>(进程内 TCP 自连)
├── Dioxus UI(托盘 + 设置页 + 鼠标跟随指示器)
└── 触摸板启用/禁用(PowerShell PnP 设备操作)
```

## 目录结构

| 路径 | 用途 |
|------|------|
| `kanata/enable-touchpad.kbd` | kanata 层配置(编译期嵌入二进制) |
| `../src/bin/enable-touchpad/` | 应用代码:`main` / `app`(UI)/ `config` / `kanata_embed` / `signal` / `touchpad` / `tray` |

应用只在 Windows 下编译;其他目标编译为占位桩,保证仓库的 Linux 门禁不受影响。

## Windows 部署步骤

1. **构建或下载** exe(Windows 上 `cargo build --release`,或取 CI 的
   test-build 产物)。
2. **以管理员身份运行**(启用/禁用设备是系统级操作)。内嵌的 kanata 配置会在
   首次启动时写入 `%APPDATA%\enable-touchpad\kanata.kbd`。
3. 按住 `CapsLock` → 触摸板打开,鼠标旁出现蓝色提示条,`Q`/`W`/`E` 变为
   鼠标按键;松开 → 全部还原,触摸板关闭。

**无需单独安装 kanata,也无需内核驱动**——kanata 的 Windows 默认模式走低级
键盘钩子。**不要同时再运行一个外部 kanata**(会造成双重按键捕获和 TCP 端口
冲突)。

## 设置页说明

- **触摸板状态** + 手动启用/禁用/刷新按钮(不依赖 kanata 也能用,方便先验证权限)。
- **信号源**:TCP(`LayerChange` 流,推荐)或 F24 按键事件。
- **总开关 / 指示器**:功能总开关与鼠标提示开关,含预览按钮。
- **应用设置 / 保存配置**:运行时即时生效;持久化到
  `%APPDATA%\enable-touchpad\config.json`。

## Demo 局限

- 触摸板启停通过调用 PowerShell 实现(`Disable-PnpDevice`/`Enable-PnpDevice`,
  按友好名匹配 `touchpad|触摸板`)—— 属于演示级做法;正式版应使用 CfgMgr32
  并配合提权辅助进程。
- 设备友好名需包含 "touchpad"/"触摸板";PS/2 触摸板的 OEM 名称可能需要扩展
  匹配规则。
- UI 渲染在系统 WebView 中(dioxus desktop);纯 GPU 原生渲染(Blitz)尚未成熟。
- TCP 模式下 F24 组合键会发出但无任何绑定,不会影响系统。
