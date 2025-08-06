<div align=center>
<img src="assets/transparent_logo.png" width="120" alt="BibCiTeX">
<h1>BibCiTeX</h1>
<p align="center">
    <img src="assets/BibTeX.png" width="60">  文献快捷引用工具
</p>
</div>

## 简介

BibCiTeX 是一个使用 🦀 Rust 和 [<img src="assets/dioxus.svg" width="15"> Dioxus](https://dioxuslabs.com) 框架开发的跨平台 <img src="assets/BibTeX.png" width="60"> 文献快捷引用工具。

### 文献类型
- [x] Article
- [x] Book
- [x] Thesis(PhDThesis && MastersThesis)
- [ ] Booklet
- [ ] InBook
- [ ] InCollection
- [ ] Manual
- [ ] Misc
- [ ] Proceedings
- [x] TechReport
- [x] InProceedings
- [ ] Unpublished


## 开发路线图
### 进行中
- [x] 文献库删除功能
- [ ] 完整的搜索功能优化
- [ ] 完善文献分类和标签系统

### 计划中
- [ ] Spotlight 风格的全局搜索助手
- [ ] macOS 系统级无焦点窗口实现
- [ ] 快捷键自定义设置

### UI/UX 改进
- [ ] 完整的 UI 设计系统
- [ ] 自定义主题支持
- [ ] 更好的响应式设计

## 第三方代码版权声明 (Third-Party Code Attribution)
### [src/platforms/macos](./src/platforms/macos)
- **来源(Source)**: [ahkohd/tauri-nspanel](https://github.com/ahkohd/tauri-nspanel) (v2.1)
- **作者(Author)**: Victor Aremu (ahkohd)
- **许可协议(License)**: [MIT](https://github.com/ahkohd/tauri-nspanel/blob/v2.1/LICENSE_MIT) OR [MIT](https://github.com/ahkohd/tauri-nspanel/blob/v2.1/LICENSE_MIT)/[Apache 2.0](https://github.com/ahkohd/tauri-nspanel/blob/v2.1/LICENSE_APACHE-2.0)
- **用途(Usage)**: 为 Dioxus 框架适配 macOS NSPanel 功能 (Adapted macOS NSPanel functionality for Dioxus framework)
- **版权声明(Copyright)**:
  ```
  Copyright (c) 2023 - Present Victor Aremu
  ```
- **主要修改(Key Modifications)**:
  - 从 Tauri 框架适配为 Dioxus 框架 (Adapted from Tauri framework to Dioxus framework)
  - 移除 Tauri 特定的运行时集成 (Removed Tauri-specific runtime integration)

详细的归属信息请参阅 [NOTICE](./NOTICE) 文件 (For detailed attribution information, please refer to the [NOTICE](./NOTICE) file)。

## 许可协议

本项目采用双重许可协议，您可以选择其中任意一种：

* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) 或 https://www.apache.org/licenses/LICENSE-2.0)
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) 或 https://opensource.org/licenses/MIT)

### 贡献声明
除非您明确声明，否则根据 Apache-2.0 许可协议的定义，您有意提交的任何贡献都将按照上述双重许可协议进行许可，不附加任何额外条款或条件。
