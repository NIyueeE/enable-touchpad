# Windows 可行性 Demo

> English | [README.md](README.md)

`enable-touchpad` 设想在 Windows 上的概念验证:

- **kanata** 负责键盘:按住 `CapsLock` 激活 `mouse` 层(`Q`/`W`/`E` → 鼠标
  左/中/右键,`Left Alt` → `CapsLock`),同时在按住期间保持输出一个系统默认
  无绑定的组合键 `Ctrl+Win+F24`。
- **伴随应用**(`src/bin/enable-touchpad`,Dioxus 构建)接收层信号,在
  **按住 CapsLock 期间启用触摸板、松开时禁用**,在鼠标位置显示可穿透点击的
  激活提示,并提供系统托盘图标和极简设置页。

```
按住 CapsLock ──▶ kanata 层 "mouse" ──▶ 信号 ──▶ Dioxus 应用
   (Q/W/E = 鼠标按键)                  TCP LayerChange      │
                                        或 F24 按下/释放      ├─▶ 启用触摸板
                                                              ├─▶ 鼠标处指示器
松开 CapsLock ──▶ 层还原 ─────────────────────────────────────┴─▶ 禁用触摸板
```

## 目录结构

| 路径 | 用途 |
|------|------|
| `kanata/enable-touchpad.kbd` | kanata 层配置(已用 `kanata --check` 验证) |
| `../src/bin/enable-touchpad/` | 应用代码:`main` / `app`(UI)/ `config` / `signal` / `touchpad` / `tray` |

应用只在 Windows 下编译;其他目标编译为占位桩,保证仓库的 Linux 门禁不受影响。

## Windows 部署步骤

1. **安装 kanata 与 Interception 驱动** —— 从
   <https://github.com/jtroo/kanata/releases> 获取 kanata,然后安装其在
   Windows 上依赖的 [Interception 驱动](https://github.com/oblitum/Interception)
   并重启。
2. **启动 kanata**(二选一信号源):
   - TCP 模式:`kanata -c enable-touchpad.kbd -p 5829`
   - F24 模式:`kanata -c enable-touchpad.kbd`
3. **以管理员身份构建并运行应用**(启用/禁用设备是系统级操作):

   ```powershell
   cargo run --bin enable-touchpad
   ```

4. 按住 `CapsLock` → 触摸板打开,鼠标旁出现蓝色提示条,`Q`/`W`/`E` 变为
   鼠标按键;松开 → 全部还原,触摸板关闭。

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
