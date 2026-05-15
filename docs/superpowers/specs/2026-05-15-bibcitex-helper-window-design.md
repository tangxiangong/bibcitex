# BibCiTeX Helper 窗口设计

## 范围

本文只重新设计 BibCiTeX 的 helper 窗口。

本文会覆盖总 UI 设计文档中的 helper 行为描述，但继续遵守同一套视觉系统：

- 浅色模式统一使用 Catppuccin `latte`。
- 深色模式统一使用 Catppuccin `mocha`。
- helper 跟随全局主题，不单独锁定为深色。
- 组件颜色使用 daisyUI 语义类和 Catppuccin 主题 token。
- 图标必须来自 `public/icons/` 下的静态 SVG；不允许 emoji，也不允许运行时 icon 库。

helper 仍然是类似 Spotlight 的引用命令面板，但首次打开、重新选择文献库、固定宽度、自动高度和虚拟滚动在本文中重新明确。

## 产品定位

helper 是最快的引用插入路径：

1. 用户通过全局快捷键或主窗口操作打开 helper。
2. 如果这是当前应用会话中第一次打开 helper，用户先选择文献库。
3. 用户在选中的文献库内搜索文献。
4. 用户用键盘或鼠标选中结果。
5. 按下 `Enter` 后，BibCiTeX 复制并粘贴 cite key 到上一个应用，然后关闭 helper。

主窗口负责添加、删除、浏览和详细检查文献库。helper 负责快速选择和快速粘贴。

## 状态模型

helper 有两个主状态：

- `library-selection`：显示可搜索的文献库列表。
- `reference-search`：在当前选中的文献库中搜索文献。

应用在当前会话中维护一个内存态的 `activeHelperBibliography`。持久化的 helper 文献库可以作为排序提示，但不能跳过“首次打开必须选择文献库”的要求。

### 首次打开

当前应用会话中第一次打开 helper 时：

- 必须显示 `library-selection`。
- 输入框立即聚焦。
- 查询为空时显示全部已配置文献库。
- 文献库排序规则：
  1. 持久化/default helper 文献库优先，如果它仍然存在。
  2. 最近更新的文献库优先。
  3. 名称排序作为稳定兜底。
- 默认高亮第一条可选文献库。
- `Enter` 选择当前高亮文献库。
- 点击文献库也选择该文献库。
- 选择后加载文献库，将其设为当前会话的 active helper bibliography，清空查询，并切换到 `reference-search`。

如果没有任何文献库：

- 显示紧凑的中文空状态。
- 如果当前命令面板中实现了跳转能力，应提供明确的中文操作入口，引导用户到主窗口添加 `.bib` 文件。
- 不显示空白面板。

### 后续打开

用户在当前应用会话中选择过一次文献库之后：

- 后续打开 helper 可以直接进入 `reference-search`。
- 输入框立即聚焦。
- 查询默认为空。
- 查询为空时只显示单行搜索栏；不显示结果区域、空状态插画或“开始输入”提示块。

如果 active helper bibliography 无法加载：

- 回退到 `library-selection`。
- 行内显示加载错误。
- helper 保持可用。

## 重新选择文献库

helper 头部只保留一个输入框和右侧紧凑文献库 chip。

当已有 active bibliography 时：

- 右侧 chip 显示当前文献库名称。
- 点击 chip 切换到 `library-selection`。
- 输入框保持聚焦。
- 默认清空查询，除非实现时明确决定复用当前查询作为文献库过滤词。
- 文献库列表中必须清楚标记当前 active bibliography。
- 选择另一个文献库后，替换 active helper bibliography，并返回 `reference-search`。

当没有 active bibliography 时：

- 右侧 chip 显示 `选择文献库`。
- 点击 chip 保持或进入 `library-selection`。

这个 chip 不是装饰元素，而是用户在不打开主窗口的情况下切换 helper 文献库的主要入口。

## 输入与键盘行为

helper 在两个状态下都只使用一个输入框。

必需键盘行为：

- `Escape`：隐藏 helper。
- `ArrowDown`：移动到下一条可见项。
- `ArrowUp`：移动到上一条可见项。
- `Enter` 在 `library-selection` 中：选择当前高亮文献库。
- `Enter` 在 `reference-search` 中：粘贴当前高亮文献的 cite key。
- 鼠标悬停可以更新当前高亮项。
- 鼠标点击执行与 `Enter` 相同的动作。

列表边界处建议循环选择，除非可访问性验证中发现问题。当前高亮行在 `latte` 和 `mocha` 中都必须清晰可见。

## 文献搜索

`reference-search` 保持命令面板模型：

- 不在 helper 中放置类型筛选或字段筛选。
- 用户输入时即时搜索。
- 搜索应覆盖 title、author、cite key、journal/venue、year、note 等现有后端能力支持的字段。
- 空查询不渲染结果列表，也不渲染下方提示区域；窗口应收缩到单行搜索框高度。
- 无结果状态显示当前查询和简短中文搜索建议。
- 粘贴成功后显示短暂本地确认，然后隐藏 helper。
- 粘贴失败时保持 helper 打开，显示失败 cite key，并提供中文 `复制` 兜底按钮。

文献结果行显示：

- 文献类型 badge。
- 标题。
- monospace cite key token。
- 作者预览。
- 年份。
- 可用时显示来源/venue。
- 一到两个可用性提示，例如 DOI、URL、PDF/文件。

结果行必须紧凑。helper 不能变成详细信息检查器。

## 主题与配色规则

helper 继承当前应用主题。

在 Catppuccin daisyUI 主题下使用语义类：

- 表面：`bg-base-100`、`bg-base-200`、`border-base-300`。
- 文本：`text-base-content`。
- 主要焦点和选中态：`bg-primary/10`、`border-primary/40`、`ring-primary/30`，或通过对比度验证的等价语义类。
- 错误：`bg-error/10`、`border-error/30`、`text-error`。
- 成功/可用/加载完成状态：`bg-success/10`、`border-success/30`，并保证文字可读。

helper 组件中不得硬编码 Catppuccin 颜色值。

文献类型和元数据属性的视觉样式应复用主工作台中的共享语义映射。颜色只作为辅助信息；每个彩色元素还必须具备文本、图标、标签或稳定位置。

## 静态 SVG 图标

helper 中所有图标都使用 `public/icons/` 下的静态 SVG。

必需图标：

- 搜索输入图标。
- 文献库 chip 图标。
- 空状态图标。
- 错误图标。
- 成功图标。
- 文献类型图标。
- cite key、作者、年份、来源、链接/文件等元数据图标。

不允许使用 emoji 图标。

## 固定宽度与自动高度

helper 宽度固定，高度根据当前内容自动调整。

### 高度

高度由以下内容决定：

- 头部高度。
- 当前可见内容高度。
- 存在时的反馈区/底部提示区高度。
- 结果区域的最大可滚动高度。

内容较少时 helper 应收缩；内容增多时 helper 应增长，直到达到最大内容高度。超过最大高度后列表内部滚动。

在 `reference-search` 中，如果查询为空且没有错误反馈，helper 高度必须只有头部搜索栏高度。不能为了展示提示而额外撑出结果区域。

### 宽度

宽度必须是固定值，不能随着输入文本、文献库名称或搜索结果内容左右跳动。

必需行为：

- Tauri helper 窗口创建时使用同一个固定宽度。
- 后续 resize 只更新高度，不更新宽度。
- 搜索输入、右侧文献库 chip、列表行文本都必须在固定宽度内通过 truncate、wrap 或紧凑布局处理。
- 空输入态也保持同一固定宽度，但高度收缩到单行搜索栏。

建议初始约束：

- 固定宽度：约 `760px`。

具体数值可以在视觉验证后微调，但最终行为必须保持“固定宽度、只自动调整高度”。

resize 失败属于非致命问题，不能阻塞搜索、选择或粘贴。

## 虚拟滚动

文献库列表和文献搜索结果列表都必须使用虚拟滚动。

要求：

- 当文献库或搜索结果数量很大时，不允许渲染全部条目。
- 只渲染可见范围和 overscan。
- 保持总 spacer 高度，使滚动条行为符合完整列表长度。
- 键盘选择必须能把当前高亮项滚动到可见区域。
- 鼠标悬停和点击必须对虚拟化行正常工作。
- resize 测量使用可见容器高度，而不是完整条目数量。
- 空状态、加载状态和错误状态不是虚拟列表行。

建议实现模型：

- 优先使用固定行高，除非确实需要可变高度。
- 文献库行高可以比文献结果行高更短。
- overscan 要足够避免键盘移动时闪烁。
- selected index 使用逻辑条目索引，而不是当前渲染行索引。

即使小文献库看起来已经足够快，虚拟滚动仍然是 helper 的验收要求。

## 加载与错误状态

文献库加载中：

- 在被选择的文献库行内显示加载标记。
- 输入框保持焦点。
- 加载过程中不关闭 helper。

文献库加载失败：

- 停留在 `library-selection`。
- 行内显示错误。
- 失败的文献库仍然可见。

搜索失败：

- 停留在 `reference-search`。
- 行内显示错误。
- 保持 active bibliography chip 可见。

粘贴失败：

- 停留在 `reference-search`。
- 显示失败的 cite key。
- 提供 `复制` 兜底按钮。
- 如果原因可能是辅助功能权限或焦点权限，应给出中文恢复提示。

所有用户可见文本必须是中文。

## 实现边界

预期实现涉及：

- `src/pages/HelperPage.tsx`
- `src/tauri.ts`
- `src-tauri/src/commands.rs`
- `src/components/reference/semantic.ts` 中的共享语义样式
- `src/constants/icons.ts` 与 `public/icons/` 中的静态 SVG

本 redesign 不应引入独立颜色系统、helper 专属主题或第二套搜索/筛选工具栏。

helper 不应改变主窗口三栏行为，除非需要显式命令打开主窗口进行恢复操作。

## 验收清单

- 当前应用会话首次打开 helper 时显示文献库选择界面。
- 选择文献库后切换到文献搜索界面。
- 后续打开 helper 使用当前 active helper bibliography，除非加载失败。
- `reference-search` 空输入态只显示单行搜索栏，不显示下方结果区域。
- 点击右侧文献库 chip 可以重新选择文献库。
- `Escape`、`ArrowUp`、`ArrowDown`、`Enter`、鼠标悬停和鼠标点击在两个状态中都正常工作。
- 搜索状态下按 `Enter` 会粘贴当前选中文献的 cite key，成功后隐藏 helper。
- 粘贴失败时 helper 保持打开，并显示中文复制兜底。
- 高度随内容自动收缩/增长。
- 宽度固定；输入、文献库 chip 和结果内容变化时窗口宽度不跳动。
- 文献库列表使用虚拟滚动。
- 文献结果列表使用虚拟滚动。
- 键盘选择能让虚拟化条目滚动到可见区域。
- helper 在浅色模式跟随 `latte`，深色模式跟随 `mocha`。
- helper 组件不硬编码 Catppuccin 颜色值。
- helper 图标不使用 emoji 或运行时 icon 库组件。
- 所有 helper 用户可见文本均为中文。
- 实现后 `bun run check` 通过。
- 实现后 `bun run build` 通过。
