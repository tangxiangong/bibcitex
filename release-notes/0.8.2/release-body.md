## 简体中文

# BibCiTeX 0.8.2

## 引用格式与批量复制

- 新增全局引用设置，可选择 key、LaTeX 或 Typst 格式，并记忆引用指令和多选输出方式。
- LaTeX 支持 `\cite`、natbib 的 `\citep` / `\citet`，以及 biblatex 的 `\autocite`、`\parencite`、`\textcite`、`\footcite`；Typst 支持 `@key`、`#cite` 和正文引用。
- 主窗口和托盘新增“引用”操作，支持单篇及多篇文献复制；多选按当前结果列表顺序输出。Helper 直接粘贴和粘贴失败后的复制也使用同一引用格式。
- 多选可选择“合并”或“逐项”输出；key 格式始终以逗号连接。BibTeX 源码复制仍保留原始内容。

在设置中选择引用格式后，合并模式下两篇文献的输出示例：

| 格式 | 输出示例 |
| --- | --- |
| key | `Knuth1984,Lamport1994` |
| LaTeX / `\cite` | `\cite{Knuth1984,Lamport1994}` |
| Typst / `@key` | `@Knuth1984 @Lamport1994` |

## 多选与标签

- 主窗口和托盘的多选操作改为紧凑的横向浮动工具栏，可直接引用、批量修改标签、全选、清空或结束多选。
- 再次点击多选按钮或按 Esc 可退出多选；编辑框、弹出菜单和中文输入法仍优先处理自己的 Esc 操作。
- 文献详情直接展示全部已分配标签，以带颜色标记的标签胶囊排列，并保留独立的添加入口；修改标签后保持当前文献选择，避免异步刷新切换到旧结果。

## 修复

- Helper 粘贴后尽量恢复原先的剪贴板内容，并避免覆盖粘贴期间新复制的内容。
- 修复托盘窗口打开后再次点击图标却立即重开的情况，现在可正常点击收起。

## macOS

- 修复主窗口和托盘中 Shift / ⌘ 点选无法直接进入多选，以及选中状态被旧更新覆盖的问题。
- Helper 仅在选择两篇及以上文献时显示已选数量，减少单选时的提示。
- 标签编辑弹窗紧邻主窗口、托盘和 Helper 的列表标签区域或详情添加按钮显示，切换文献时关闭旧弹窗。

## Windows

- 缩小并统一设置下拉框宽度；引用指令选中后简洁显示命令，展开列表时保留说明。
- 修复托盘图标双击触发重复开关的问题。

## English

# BibCiTeX 0.8.2

## Citation formats and batch copying

- Add global citation settings for key, LaTeX, and Typst formats, with saved citation commands and multiple-citation output preferences.
- Support LaTeX `\cite`, natbib's `\citep` / `\citet`, and biblatex's `\autocite`, `\parencite`, `\textcite`, and `\footcite`; Typst supports `@key`, `#cite`, and prose citations.
- Add a Cite action to the main and tray windows for copying one or more references in result-list order. Helper direct paste and copy fallback after a failed paste use the same citation format.
- Choose merged or individual output for multiple citations; key output always uses commas. Copying BibTeX source preserves the original content.

After choosing a citation format in Settings, merged output for two references looks like this:

| Format | Example output |
| --- | --- |
| key | `Knuth1984,Lamport1994` |
| LaTeX / `\cite` | `\cite{Knuth1984,Lamport1994}` |
| Typst / `@key` | `@Knuth1984 @Lamport1994` |

## Multiple selection and tags

- Replace selection menus in the main and tray windows with compact horizontal floating toolbars for citations, batch tag editing, selecting all, clearing, and ending selection.
- Click the selection button again or press Esc to leave selection mode. Editors, popup menus, and IME composition retain priority for their own Escape handling.
- Show all assigned tags directly in reference details as capsules with color markers and a separate add control. Keep the current reference selected after tag changes and prevent stale asynchronous refreshes from replacing current results.

## Fixes

- Restore previous clipboard content after Helper paste where supported, without overwriting content copied during the paste.
- Fix tray windows immediately reopening when their icon is clicked again; a second click now closes the window.

## macOS

- Fix Shift / ⌘ clicks failing to enter multiple selection directly in the main and tray windows, and prevent stale updates from overriding selection.
- Show the Helper selection count only when two or more references are selected.
- Anchor tag editors beside list tag areas in the main window, tray, and Helper, or beside the detail add button; close stale editors when the selected reference changes.

## Windows

- Make Settings dropdowns narrower and consistent. Show only the selected citation command while retaining descriptions in the expanded list.
- Prevent double-clicking the tray icon from toggling the window repeatedly.
