## 简体中文

# BibCiTeX 0.8.0

## 搜索更灵活

- 普通搜索支持多个关键词同时匹配，可跨标题、作者等默认字段查找；改善 Unicode、重音、标点、前缀与部分匹配，并为适用的词提供有限拼写容错，优先展示正常命中。
- 新增高级搜索：字段、连续短语、年份比较和范围、文献类型、标签，以及 AND / OR / NOT 与括号分组。也可在主窗口或托盘窗口的条件编辑器中组合条件。
- 普通与高级搜索分别保留输入草稿；改进结果排序与缓存，减少反复检索开销。

## 高级搜索

在主窗口或托盘窗口打开“高级搜索”开关（两处默认关闭，分别记忆）；Helper 默认使用高级搜索。普通模式不会解释 `author:`、`OR` 等高级语法。

| 目的                           | 查询示例                          |
| ------------------------------ | --------------------------------- |
| 同时包含两个关键词             | `graph network`                   |
| 连续短语                       | `"machine learning"`              |
| 作者及年份下限                 | `author:Knuth year:>=2020`        |
| 标题关键词并排除综述           | `title:graph -survey`             |
| 两个关键词任选其一，再限定作者 | `(graph OR network) author:Smith` |
| 年份范围，包含两端             | `year:2020..2024`                 |
| 期刊论文或书籍                 | `type:(article OR book)`          |
| 同时具有两个标签               | `tag:待读 tag:数学`               |
| 任一标签，排除已读             | `tag:(数学 OR 算法) -tag:已读`    |
| 完整引用键                     | `key:Smith2024`                   |

- 空格表示 AND。布尔运算符必须大写；优先级为括号 → `NOT` / `-` → AND → OR。`-survey` 等价于 `NOT survey`，减号后不留空格。
- 字段名使用英文，冒号后紧接值：写 `author:Knuth`，不要写 `author: Knuth`。`title:(graph neural)` 将两个词都限定在标题。
- 还可使用 `editor`、`journal`、`booktitle`、`abstract`、`note`、`publisher`、`school`、`institution`、`organization`、`doi`、`isbn`；`intitle` 是 `title` 的别名，`citekey` 是 `key` 的别名。引用键、DOI、ISBN 按完整标识符匹配。
- 年份支持 `>`、`>=`、`<`、`<=`、`=` 和范围；`year:2020..` 表示 2020 年及以后，`year:..2024` 表示 2024 年及以前。
- 用英文双引号包围短语或含空格的标签，例如 `tag:"machine learning"`。标签按完整名称匹配；不存在的标签不会自动创建。
- 未闭合引号、括号或缺少字段值时，补全输入后再搜索；错误或未完成查询不能继续操作旧结果。搜索字面运算符可加引号，例如 `"AND"`、`"title:graph"`。

### Helper 快捷筛选

1. 在输入末尾的空白边界直接键入 `@` 选择文献类型、`$` 选择字段、`#` 选择已有标签。
2. 在小输入框中筛选候选名称，用上下键选择、Enter 或点击确认；类型和标签直接形成条件胶囊。
3. 字段另有可选值框：例如选择 year，输入 `2026` 并按 Enter，即生成 `year:2026`。留空按 Enter、Tab 或 Esc，则将该字段设为后续关键词的搜索范围。
4. 继续输入关键词或添加条件；尾框光标在开头且没有选区时，Backspace 可删除前一个胶囊。Esc 可取消候选。

中文输入法选字优先；粘贴文本、邮箱或 `C#` 中的符号不会触发候选。完整规则见 [高级搜索指南](https://github.com/tangxiangong/bibcitex/blob/v0.8.0/README-zh.md#高级搜索使用方法)。

## 标签与多选

- 为文献添加彩色标签，支持创建、改名、改色、合并、删除和失效关联恢复；标签跨文献库共享，移动或重新定位文献库后保留稳定身份，不改写 `.bib` 文件。
- 主窗口和托盘支持多选与批量添加、移除、创建并添加标签，区分全部或部分关联，整批校验后写入。
- Helper 普通点击直接粘贴；⌘（Windows 为 Ctrl）点选增减选择，Shift 点选或 Shift+方向键扩选，⌘/Ctrl+A 全选当前结果，Enter 按列表顺序将所选引用键以逗号连接后粘贴。
- 显示已选数量和可识别的粘贴目标应用；搜索、筛选或切换文献库后清空选择，避免操作过期结果。
- 标签备份请同时保留配置目录中的 `setting.json` 和 `search-metadata.json`；普通 BibTeX 导入导出不携带标签。

主窗口/托盘的批量复制及 LaTeX、Typst 引用格式预设尚未提供。


**macOS**

## macOS

- 修复列表选中高亮闪烁和滚动内容边距问题，改善窄窗口布局、工具栏与详情面板。
- 改善托盘弹窗保持、搜索焦点恢复和多选状态，优化 Helper 滚动、底部操作区及粘贴操作的辅助功能支持。
- 修复附件路径中冒号和转义分隔符的解析。

---



**Windows**

## Windows

- 对齐主窗口、托盘的搜索控件与工具栏按钮，改善候选浮层定位、滚动和键盘选择。
- 调整 Helper 底部操作区，避免提示覆盖搜索结果，保持空闲面板紧凑。
- 修复过期异步粘贴请求的取消处理，避免旧请求继续执行。

---



## English

# BibCiTeX 0.8.0

## More flexible search

- Ordinary search matches multiple terms across default fields such as title and author. Improve Unicode, accent, punctuation, prefix and substring matching, with limited spelling tolerance for eligible words and normal matches ranked first.
- Add advanced search with fields, consecutive phrases, year comparisons and ranges, reference types, tags, AND / OR / NOT, and parentheses. Conditions can also be composed in the main or tray window's condition editor.
- Keep separate drafts for ordinary and advanced search, and improve ranking and caching for repeated searches.

## Using advanced search

Enable advanced search in the main or tray window (off by default and remembered separately). Helper uses advanced search by default. Ordinary mode does not interpret syntax such as `author:` or `OR`.

| Goal                                    | Example                              |
| --------------------------------------- | ------------------------------------ |
| Match both terms                        | `graph network`                      |
| Match a consecutive phrase              | `"machine learning"`                 |
| Author and minimum year                 | `author:Knuth year:>=2020`           |
| Title term, excluding surveys           | `title:graph -survey`                |
| Either term, with an author restriction | `(graph OR network) author:Smith`    |
| Inclusive year range                    | `year:2020..2024`                    |
| Articles or books                       | `type:(article OR book)`             |
| Both tags                               | `tag:reading tag:math`               |
| Either tag, excluding read items        | `tag:(math OR algorithms) -tag:read` |
| Full citation key                       | `key:Smith2024`                      |

- Spaces mean AND. Boolean operators must be uppercase. Precedence is parentheses → `NOT` / `-` → AND → OR. `-survey` equals `NOT survey`; do not put a space after the minus sign.
- Use English field names and place the value immediately after the colon: `author:Knuth`, not `author: Knuth`. `title:(graph neural)` restricts both terms to the title.
- Other fields include `editor`, `journal`, `booktitle`, `abstract`, `note`, `publisher`, `school`, `institution`, `organization`, `doi`, and `isbn`. `intitle` aliases `title`; `citekey` aliases `key`. Citation keys, DOIs and ISBNs match complete identifiers.
- Years support `>`, `>=`, `<`, `<=`, `=`, and ranges. `year:2020..` means 2020 onward; `year:..2024` means up to and including 2024.
- Use ASCII double quotes for phrases or tags containing spaces, such as `tag:"machine learning"`. Tags match complete names; unknown tags are not created automatically.
- Complete missing values, quotes or parentheses before searching. Invalid or incomplete queries disable actions on stale results. Quote literal operators, for example `"AND"` or `"title:graph"`.

### Helper quick filters

1. Type `@` for reference types, `$` for fields, or `#` for existing tags at a whitespace boundary at the end of the input.
2. Filter candidate names in the small input, navigate with arrow keys, and confirm with Enter or a click. Types and tags immediately become condition capsules.
3. Fields have a separate optional value input: choose year, enter `2026`, and press Enter to create `year:2026`. Leave it empty and press Enter, Tab or Esc to use that field as the scope for subsequent keywords instead.
4. Continue typing keywords or adding conditions. At the start of the trailing input with no selection, Backspace removes the preceding capsule. Esc cancels candidates.

IME composition takes priority. Pasted text, email addresses and symbols inside words such as `C#` do not trigger candidates. See the [complete advanced search guide](https://github.com/tangxiangong/bibcitex/blob/v0.8.0/README.md#advanced-search-guide).

## Tags and multiple selection

- Add colored tags with creation, renaming, recoloring, merging, deletion and broken-association recovery. Tags are shared across libraries and retain stable identities when libraries move or are relocated, without modifying `.bib` files.
- Select multiple references in the main and tray windows to add, remove, or create and assign tags in one batch, with full/partial association states and validation before writing the whole batch.
- In Helper, a normal click pastes immediately. ⌘-click (Ctrl-click on Windows) toggles selection; Shift-click or Shift+arrows extends it; ⌘/Ctrl+A selects all current results. Enter pastes selected citation keys in list order, joined with commas.
- Show selected counts and the paste destination application when identifiable. Searching, filtering or switching libraries clears selection to prevent stale actions.
- Back up both `setting.json` and `search-metadata.json` from the configuration directory to preserve tags. Ordinary BibTeX import/export does not carry tags.

Batch copying from the main/tray windows and LaTeX or Typst citation-format presets are not yet available.


**macOS**

## macOS

- Fix selection highlight flashes and scroll content insets; improve narrow-window layouts, toolbars and detail panes.
- Improve tray sheet retention, search focus restoration and selection state, plus Helper scrolling, footer layout and accessible paste activation.
- Fix attachment-path parsing for colons and escaped delimiters.

---



**Windows**

## Windows

- Align search controls and toolbar buttons in the main and tray windows; improve candidate popup placement, scrolling and keyboard selection.
- Adjust Helper's footer so hints do not overlap results, while keeping the idle panel compact.
- Fix cancellation of stale asynchronous paste requests so obsolete requests do not continue executing.

---
