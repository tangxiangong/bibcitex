# BibCiTeX macOS 原生 Swift Helper 设计

## 范围

本文定义 BibCiTeX helper 在 macOS 上从 `tauri-nspanel` 和 `/helper`
webview 迁移到完全原生 Swift 的设计。

这次改造只覆盖 macOS helper：

- macOS 使用 SwiftUI 实现 helper UI。
- AppKit 只提供 `NSPanel` 外壳和窗口生命周期能力。
- Windows 和 Linux 继续使用现有 `/helper` Tauri webview。
- `/helper` 页面保留为非 macOS 平台 fallback。
- macOS 代码必须明确避免误创建 `/helper` webview helper。
- 移除 `tauri-nspanel` 依赖、插件初始化和相关面板代码。

## 目标

这是一次功能移植，不是重新定义 helper 产品形态。原生 Swift helper
必须保持现有功能：

1. 首次打开选择文献库。
2. 后续打开记住当前会话中的 helper 文献库。
3. 输入搜索文献库或引用。
4. 支持键盘上下选择、`Enter` 粘贴、`Escape` 隐藏。
5. 支持鼠标 hover 高亮和点击选择。
6. 动态高度随内容变化。
7. 粘贴失败时保留 cite key，并提供复制兜底按钮。
8. 无文献库时在 helper 内显示中文空状态。
9. 主题和主应用保持一致。
10. 数学公式正确渲染。

## 架构

采用 Xcode Swift helper library + Cargo/Tauri 链接的架构。

`macos/NativeHelper/` 放置 Xcode 工程，负责 macOS 原生 helper：

- SwiftUI 实现完整 helper UI。
- AppKit 创建和管理 `NSPanel`。
- Cargo/Tauri 仍然构建最终应用，并在 macOS 构建时链接 native helper。
- Rust 仍然拥有设置、BibTeX 解析、搜索、剪贴板和跨应用粘贴能力。
- 主窗口、托盘和全局快捷键仍然先进入 Rust 命令层。
- macOS 的 Rust 命令层转发到 Swift native helper。
- 非 macOS 平台继续创建现有 Tauri webview helper。

架构边界：

```text
Tauri main window / tray / global shortcut
        |
        v
Rust command layer
        |
        | macOS only: C ABI calls
        v
Swift NativeHelper library
        |
        +-- AppKit NSPanel shell
        +-- SwiftUI HelperView
        +-- Swift ViewModel
```

AppKit 只处理 SwiftUI 无法稳定表达的窗口能力：

- 浮动 panel。
- 失焦隐藏。
- 置顶层级。
- 跨 Space 行为。
- 动态尺寸。
- 聚焦输入框。

SwiftUI 是 helper UI 的唯一实现来源。

## Rust 和 Swift FFI

Rust 与 Swift 之间使用底层 C ABI，不使用 JSON。

### 对象生命周期

采用混合内存模型：

- 大对象使用 Rust 持有的 opaque handle。
- 短生命周期结果使用 Rust 分配的数组。
- Swift 用完短生命周期数组后必须调用对应释放函数。
- Swift 切换文献库或关闭 helper 时释放文献库 handle。

核心对象：

- `HelperLibraryHandle`：Rust 持有当前文献库的完整 `Vec<Reference>`。
- `FfiReferenceList`：搜索结果数组，短生命周期。
- `FfiBibliographyList`：文献库列表，短生命周期。
- `FfiReference`：覆盖 Rust `Reference` 全字段。

### ABI 表示规则

低层 ABI 遵循普通 C 兼容结构：

- 字符串：`ptr + len`。
- 数组：`ptr + len`。
- `Option<T>`：`has_value + value`。
- 枚举：明确 tag 字段；带 payload 的变体使用额外字段。
- `EntryType::Unknown(String)`：`kind + unknown_text`。
- `Range<u32>`：`has_value + start + end`。
- Rust 分配的字符串和数组必须由 Rust 释放。

### Reference 覆盖范围

FFI 覆盖当前 Rust `Reference` 的完整字段，而不是只传 helper 当前显示字段。
SwiftUI 第一阶段只展示现有 helper 需要的字段，但 ABI 要能承载完整数据：

- cite key。
- source。
- entry type。
- author。
- title。
- journal。
- year。
- full journal。
- volume。
- number。
- pages。
- note。
- doi。
- mrclass。
- publisher。
- isbn。
- series。
- url。
- file。
- abstract。
- edition。
- issue。
- book pages。
- school。
- address。
- book title。
- editor。
- month。
- organization。
- institution。
- eprint。
- archive prefix。
- arxiv primary class。
- how published。

### Chunk 和数学公式

`Chunk[]` 不在 Rust 中拍平成字符串。FFI 保留 chunk 类型和文本：

- normal。
- verbatim。
- math。

SwiftUI 负责所有 helper 文本中的数学公式渲染，覆盖 inline math 和 display
math。允许引入 Swift 原生数学渲染依赖。

## SwiftUI 行为

SwiftUI helper 复刻现有 `/helper` 行为。

### 状态

helper 有两个主要状态：

- 文献库选择。
- 引用搜索。

当前 helper 文献库保存在 Rust 进程内存，不持久化到设置文件。

首次打开时：

- 从 Rust FFI 读取文献库列表。
- 文献库按 `updatedAt` 降序。
- 如果没有文献库，显示“未找到文献库，请先在主页添加文献库”。
- 不自动打开主窗口。

后续打开时：

- 如果 Rust 内存中已有当前 helper 文献库，则直接进入引用搜索。
- 如果当前库无法加载，则回到文献库选择状态并显示中文错误。

### 输入

保留一个输入框：

- 文献库选择状态下筛选文献库。
- 引用搜索状态下搜索引用。
- 点击当前文献库 chip 后回到文献库选择状态。

### 搜索

搜索逻辑复用 Rust `search_references`，Swift 不重写搜索，也不重排搜索结果。

性能策略：

- 输入使用短 debounce。
- 每次搜索带版本号。
- 过期搜索结果直接丢弃。
- 加载文献库、搜索和粘贴都不能阻塞 SwiftUI 主线程。

### 列表

helper 需要支持大型文献库：

- 文献库列表和搜索结果列表使用 SwiftUI 懒加载或虚拟化思路。
- UI 不一次性渲染所有行。
- 支持键盘选择和鼠标 hover。

### 粘贴

粘贴继续复用现有 Rust `xpaste` 流程：

1. Rust 复制 cite key 到剪贴板。
2. Rust 聚焦上一个应用。
3. Rust 触发粘贴。

如果粘贴失败：

- helper 保持打开。
- 显示中文错误。
- 保留失败 cite key。
- 提供复制兜底按钮。

## 视觉和主题

SwiftUI helper 视觉尽量贴近现有 `/helper`：

- Spotlight 式浮动面板。
- 圆角。
- 半透明背景。
- 阴影。
- 细边框。
- 顶部搜索输入。
- 右侧当前文献库 chip。
- 文献库列表和引用列表的信息层级与现有 UI 接近。
- 选中态、hover 态和错误态保持清晰。

具体宽高数值可以重新校准，但必须保持动态高度：

- 空查询时只显示输入栏和必要错误状态。
- 内容增加时面板高度增长。
- 高度达到上限后列表内部滚动。
- 宽度保持固定的 Spotlight 面板感。

主题要求：

- helper 打开时读取当前主应用主题。
- 主窗口运行中切换主题时同步到 Swift helper。
- 保持现有 latte/mocha 视觉一致性。
- SwiftUI 不使用独立主题。

## 平台分支

macOS：

- Rust helper 命令只调用 native helper。
- 不创建 `/helper` webview。
- 不初始化 `tauri-nspanel`。
- 不依赖 `tauri-nspanel`。

Windows/Linux：

- 继续使用当前 `/helper` 页面。
- 继续通过 Tauri webview helper 实现快捷助手。
- 不受 macOS 原生迁移影响。

## 构建

macOS native helper 由 `macos/NativeHelper/` 下的 Xcode 工程管理源码。

Cargo/Tauri 构建仍然是最终入口：

- `bun run tauri:dev` 仍然启动开发应用。
- `bun run tauri:build` 仍然构建生产应用。
- macOS 构建时链接 Xcode helper library。

构建脚本必须清楚表达：

- 只在 macOS 构建 Swift helper。
- 非 macOS 构建不依赖 Xcode。
- Swift helper 构建失败时让 Cargo 构建失败。

## 测试和验收

### 自动化检查

Rust 侧：

- FFI 字符串分配和释放。
- FFI 数组分配和释放。
- `Option`、枚举 tag、`Range` 表示。
- `Chunk[]` 传递和释放。
- 完整 `Reference` 传递和释放。
- 文献库 handle 加载和释放。
- 搜索结果生命周期。
- `search_references` 返回顺序不被 Swift 层改变。

命令：

```bash
cd src-tauri && cargo fmt
cd src-tauri && cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cd src-tauri && cargo test
bun run check
```

### 手动 macOS 验收

- 全局快捷键打开 helper。
- 托盘打开 helper。
- 主窗口按钮打开 helper。
- 首次打开选择文献库。
- 后续打开直接进入当前库搜索。
- 点击文献库 chip 重新选择文献库。
- 输入筛选文献库。
- 输入搜索引用。
- 搜索结果顺序与 Rust 返回一致。
- 上下键选择。
- Enter 自动粘贴 cite key。
- Escape 隐藏。
- 鼠标 hover 和点击选择。
- 动态高度随内容变化。
- 大型文献库仍然流畅。
- 主应用主题切换后 helper 同步更新。
- inline 和 display math 正确渲染。
- 粘贴失败时显示中文错误和复制兜底按钮。

## 非目标

这次不做以下事情：

- 不重写主窗口。
- 不删除 Windows/Linux 的 `/helper` fallback。
- 不在 Swift 中重写搜索算法。
- 不把当前 helper 文献库持久化到设置文件。
- 不把整个 macOS app 壳迁移到 Xcode。
- 不引入 JSON 作为 Rust/Swift 数据边界。

## 已确认约束

- 任何重要决定都先和用户确认。
- SwiftUI 是 UI 主体。
- AppKit 只做 panel 外壳。
- 使用 Xcode 工程管理 native helper library。
- 最终仍由 Cargo/Tauri 构建应用。
- FFI 使用底层 C ABI。
- 完整 `Reference` 和 `Chunk[]` 传给 Swift。
- 数学公式由 SwiftUI 侧正确渲染。
- 保持现有 helper 功能。
- 复用 Rust 搜索逻辑。
- 当前 helper 文献库只保存在 Rust 内存。
- 使用 handle + 短生命周期数组的混合内存模型。
- 主题与主应用保持一致。
- macOS 明确不再创建 `/helper` webview helper。
- 移除 `tauri-nspanel`。
