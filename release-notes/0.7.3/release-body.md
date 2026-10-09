## 简体中文

# BibCiTeX 0.7.3

本次发布提供 macOS 和 Windows 版本，不提供 Linux 版本。

## 通用更新

- 文献库文件丢失时，将文献库标记为“不可用”，保留原有默认文献库设置，避免启动时弹出读取错误；快捷引用会引导重新选择可用文献库。
- 文献详情面板直接显示所选文献的标题，BibTeX 内容默认展开，减少查看步骤。
- 将引用键和 BibTeX 的复制按钮移至对应字段旁；URL、DOI 以及主窗口中的文件路径可直接点击打开。
- 统一主窗口、托盘和快捷引用的圆角选中样式，优化快捷引用中的“粘贴引用键”和“切换文献库”操作布局。
- 将文献详情开关移至工具栏右侧，便于识别和操作。
- 更新公告支持按平台筛选：应用内展示通用更新及当前平台更新，GitHub 展示完整公告。


**macOS**

## macOS

- 优化快捷引用和托盘的玻璃材质、圆角、阴影及淡入淡出效果，并保留较旧 macOS 版本的材质兼容。
- 修复文献库操作菜单图标缺失的问题。
- 调整主窗口最小宽度，避免恢复较窄窗口且显示文献详情时触发布局循环和崩溃。

---



**Windows**

## Windows

- 修复主窗口和快捷引用搜索框的文本指针与输入光标显示问题。
- 统一搜索输入框样式，改善半透明背景下的聚焦显示，并适配高对比度模式。
- 优化托盘详情面板和工具栏样式，调整主窗口与托盘的滚动条显示。

---



## English

# BibCiTeX 0.7.3

This release is available for macOS and Windows. No Linux version is included.

## Common updates

- Mark libraries whose files are missing as unavailable while preserving the saved default library, avoiding a file-reading error at startup. Quick citation prompts you to choose an available library.
- Show the selected reference's title in the details header and keep BibTeX expanded for easier access.
- Place citation-key and BibTeX copy buttons beside their fields. Open URLs, DOIs, and file paths in the main window directly from their values.
- Unify rounded selection styling across the main window, tray, and quick citation, and refine the placement of the paste-citation-key and switch-library controls.
- Move the reference-details toggle to the right edge of the toolbar.
- Support platform-specific release notes: the app shows common updates and updates for its platform, while GitHub shows the complete announcement.


**macOS**

## macOS

- Refine glass materials, rounded corners, shadows, and fade transitions in quick citation and the tray, retaining compatible materials on older macOS versions.
- Fix missing icons in library action menus.
- Adjust the minimum main-window width to prevent layout loops and crashes when restoring a narrow window with reference details visible.

---



**Windows**

## Windows

- Fix text-pointer and insertion-caret visibility in the main-window and quick-citation search fields.
- Unify search-field styling, improve focus visibility over translucent backgrounds, and support high-contrast mode.
- Refine tray detail panels and toolbars, and adjust scrollbar visibility in the main window and tray.

---



