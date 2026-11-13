# BibCiTeX 0.8.4

<!-- platform:macos,windows -->
## 托盘与快捷键设置

- 新增托盘图标显示开关，设置会自动保存。
- 直接按键录制 Helper 快捷键，修改后立即生效，并可在同一行恢复默认快捷键；替换注册失败时保留原有快捷键。
- 将语言和主题设置归入“外观”，让设置页更紧凑。

## 依赖更新

- 更新共享核心和原生客户端依赖。
<!-- /platform -->

<!-- platform:macos -->
## macOS

- 快捷键录制控件使用系统修饰键符号。
- 更新 Sparkle，继续提供同时支持 Apple Silicon 和 Intel 的 Universal 安装包及应用归档。
<!-- /platform -->

<!-- platform:windows -->
## Windows

- 快捷键使用 Win、Ctrl、Alt 和 Shift 等 Windows 按键名称。
- 升级 Windows App SDK，并同步运行时安装和启动引导版本，避免使用旧版运行时。
<!-- /platform -->

## 发布范围

- 本次发布包含 macOS 和 Windows。Linux 发布构建继续暂停，等待原生桌面验收。
