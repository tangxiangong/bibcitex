## 简体中文

# BibCiTeX 0.8.3


**macOS / Windows**

## 嵌套标签筛选

- 主窗口和托盘支持嵌套 AND、OR、NOT 标签条件组，可与关键词、字段和文献类型筛选组合使用。
- 在紧凑的标签弹窗中编辑条件，确认后统一应用。标签合并后更新已选标签 ID，缺失标签和未完成条件组不会静默扩大结果范围。

## 更新

- 新增自动检查更新设置，并在应用操作进行期间推迟更新操作。

---



**macOS**

## macOS

- 提供同时支持 Apple Silicon 和 Intel 的 Universal 安装包及应用归档；原有分架构更新源也将接收 Universal 应用。
- 修复按 Command-H 隐藏主窗口后，打开托盘面板会重新显示主窗口的问题。

---



**Windows**

## Windows

- 在信任安装包哈希之前验证更新元数据签名，同时保留 COS 下载和 GitHub 回退。
- 新增持久化自动检查计划、退出时安装及更清晰的更新状态；报告下载超时并拒绝过期缓存安装包。
- Helper 候选项颜色随主题同步变化，放大文字时避免内容裁切。

---


## 发布范围

- 本次发布包含 macOS 和 Windows。Linux 发布构建继续暂停，等待原生桌面验收。


## English

# BibCiTeX 0.8.3


**macOS / Windows**

## Nested tag filters

- Build nested AND, OR and NOT tag groups in the main and tray windows, and combine them with keyword, field and reference-type filters.
- Edit conditions in a compact tag popover, with changes applied together when confirmed. Tag merges refresh selected IDs; missing tags and incomplete groups do not silently broaden results.

## Updates

- Add automatic update-check preferences and defer update operations while application work is active.

---



**macOS**

## macOS

- Ship one Universal installer and app archive for Apple Silicon and Intel. Existing architecture-specific update feeds receive the Universal app.
- Keep the main window hidden after Command-H when opening the tray panel.

---



**Windows**

## Windows

- Verify signed update metadata before trusting package hashes, while retaining COS delivery and GitHub fallback.
- Add persisted automatic-check scheduling, installation on exit, and clearer update status; report download timeouts and reject stale cached packages.
- Keep Helper candidate colors synchronized with theme changes and accommodate scaled text without clipping.

---


## Release scope

- This release includes macOS and Windows. Linux release builds remain disabled pending native desktop acceptance.
