# TODOList

## 新功能
-[ ] 快捷应用助手，类似于 Spotlight，可以快速搜索和将 `cite_key` 插入到激活中的编辑器贯标所在处
 -[ ] 需要注意的是，在 macOS 中，`tao` 的 `no-focused` 是不起作用的，需要参考 [tauri-plugin-spotlight](https://github.com/zzzze/tauri-plugin-spotlight) 调用系统级别的 API 实现无焦点搜索
-[ ] 设置修改，添加快捷键设置
-[ ] 增加 `\cite` 类型
-[ ] 增加文献库的删除功能
-[ ] 监测 `.bib` 文件的变化，自动更新文献库


## 优化
-[ ] 完整的搜索功能
-[ ] 文献种类分类


## UI 样式
目前没有对 UI 进行设计，将上述待办事项完成后，利用 Tailwind CSS 进行 UI 设计。
