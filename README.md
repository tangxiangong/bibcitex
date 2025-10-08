<div align="center">
  <img src="assets/transparent_logo.png" width="120" alt="BibCiTeX">

  <p>
    <img src="assets/readme/BibTeX.png" width="45"> 文献快捷引用工具
  </p>

  <p>
    <a href="https://github.com/tangxiangong/bibcitex/releases">
      <img src="https://img.shields.io/github/v/release/tangxiangong/bibcitex?style=for-the-badge&logo=github&color=blue" alt="GitHub Release">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/blob/main/LICENSE-MIT">
      <img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue?style=for-the-badge" alt="License">
    </a>
  </p>

  <p>
    <strong>快速下载</strong>
  </p>

  <p>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.5.0/BibCiTeX-v0.5.0-macos-arm64.dmg">
      <img src="https://img.shields.io/badge/macOS-Apple Silicon-000000?style=for-the-badge&logo=apple&logoColor=white" alt="macOS Apple Silicon">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.5.0/BibCiTeX-v0.5.0-macos-x86_64.dmg">
      <img src="https://img.shields.io/badge/macOS-Intel-000000?style=for-the-badge&logo=apple&logoColor=white" alt="macOS Intel">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.5.0/BibCiTeX-v0.5.0-windows-arm64.exe">
      <img src="https://img.shields.io/badge/Windows-ARM64-0078D4?style=for-the-badge&logo=windows&logoColor=white" alt="Windows ARM64">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.5.0/BibCiTeX-v0.5.0-windows-x86_64.exe">
      <img src="https://img.shields.io/badge/Windows-x86__64-0078D4?style=for-the-badge&logo=windows&logoColor=white" alt="Windows x86_64">
    </a>
  </p>
</div>

## 简介

**BibCiTeX** 是一个使用 **Rust** 和 [<img src="assets/readme/dioxus.svg" width="15"> **Dioxus**](https://dioxuslabs.com) 框架开发的现代化 <img src="assets/readme/BibTeX.png" width="20"> **BibTeX** 文献快捷引用工具。

### 核心特性

- **一键复制**: 便捷的引用复制功能
- **跨应用粘贴**: 无缝集成到您的工作流程

## 安装指南

### 下载

从 [**Release 页面**](https://github.com/tangxiangong/bibcitex/releases) 下载对应平台架构的最新版本安装包。

### macOS 安装说明

若提示 `BibCiTeX` 已损坏，请打开终端执行以下命令：

```bash
sudo xattr -dr com.apple.quarantine /Applications/BibCiTeX.app
```

> **提示**: 这是由于 macOS 的安全机制导致的，执行上述命令后即可正常使用。

### 支持的文献类型

<div align="center">

| 类型 | 状态 | 类型 | 状态 |
|:---:|:---:|:---:|:---:|
| Article | ✓ | Book | ✓ |
| Thesis | ✓ | Booklet | ✓ |
| InBook | ✓ | InCollection | ✓ |
| Misc | ✓ | TechReport | ✓ |
| InProceedings | ✓ | Manual | 进行中 |
| Proceedings | 进行中 | Unpublished | 进行中 |

</div>

> ✓ 已支持 &nbsp;&nbsp; 进行中 开发中

## 界面功能预览

<div align="center">

### 核心功能展示

| 添加 `.bib` 文件 | 文献列表 | 智能搜索 |
| :---: | :---: | :---: |
| [<img src="assets/readme/add_bib.gif" width="120" style="border-radius: 8px;">](./assets/readme/add_bib.gif) | [<img src="assets/readme/show_details.gif" width="120" style="border-radius: 8px;">](./assets/readme/show_details.gif) | [<img src="assets/readme/search.gif" width="120" style="border-radius: 8px;">](./assets/readme/search.gif) |
| *快速导入 BibTeX 文件* | *查看文献详细信息* | *实时搜索过滤* |

| 侧边详情 | 外部链接 | 复制引用 |
| :---: | :---: | :---: |
| [<img src="assets/readme/drawer.gif" width="120" style="border-radius: 8px;">](./assets/readme/drawer.gif) | [<img src="assets/readme/url.gif" width="120" style="border-radius: 8px;">](./assets/readme/url.gif) | [<img src="assets/readme/copy.gif" width="120" style="border-radius: 8px;">](./assets/readme/copy.gif) |
| *侧边栏详情展示* | *快速访问外部资源* | *一键复制引用格式* |

### 特色功能

<div style="margin: 20px 0;">
  <h4>跨应用粘贴</h4>
  <a href="assets/readme/cross_paste.gif">
    <img src="assets/readme/cross_paste.gif" alt="跨应用粘贴演示" style="max-width: 600px; border-radius: 12px; box-shadow: 0 4px 12px rgba(0,0,0,0.1);">
  </a>
  <p><em>无缝集成到您的工作流程，支持跨应用程序粘贴功能</em></p>
</div>

</div>

## 开发路线图

<div align="center">

### 进行中

| 功能 | 状态 | 描述 |
|:---|:---:|:---|
| 文献库删除功能 | ✓ | 支持删除不需要的文献条目 |
| 跨应用粘贴功能 | ✓ | 无缝集成到其他应用程序 |
| 搜索功能优化 | 进行中 | 提升搜索准确性和速度 |
| 文献分类标签系统 | 进行中 | 更好的文献组织管理 |

### 计划中

| 功能 | 优先级 | 描述 |
|:---|:---:|:---|
| macOS NSPanel 支持 | 高 | 系统级无焦点窗口实现 |
| 自定义设置 | 高 | 个性化配置选项 |

### UI/UX 改进

| 功能 | 优先级 | 描述 |
|:---|:---:|:---|
| 完整设计系统 | 高 | 统一的视觉设计语言 |
| 自定义主题支持 | 中 | 深色/浅色主题切换 |
| 响应式设计 | 中 | 适配不同屏幕尺寸 |

</div>

> 高 高优先级 &nbsp;&nbsp; 中 中优先级 &nbsp;&nbsp; ✓ 已完成 &nbsp;&nbsp; 进行中 开发中

## 第三方代码版权声明 (Third-Party Code Attribution**)

### [crates/xpaste](./crates/xpaste)
- **来源(Source)**: [EcoPasteHub/EcoPaste](https://github.com/EcoPasteHub/EcoPaste)
- **作者(Author)**: EcoPasteHub
- **许可协议(License)**: [Apache 2.0](https://github.com/EcoPasteHub/EcoPaste/blob/master/LICENSE)
- **用途(Usage)**: 实现跨应用的粘贴功能 (Cross-application paste functionality)
- **版权声明(Copyright)**:
  ```
  Copyright (c) EcoPasteHub
  ```
- **主要修改(Modifications)**:
  -  macOS: 将过时的 `objc` 和 `cocoa` 替换为 `objc2` 相关的 API (Replace deprecated `objc` and `cocoa` with `objc2` related APIs)
  - Windows: 将过时的 `winapi` 替换为 `windows-sys` 相关的 API (Replace deprecated `winapi` with `windows-sys` related APIs)
  - Linux: 移除对 Linux 平台的支持 (Remove Linux support)

### [crates/updater](./crates/updater)
- **来源(Source)**: [tauri-apps/tauri-plugion-updater](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/updater)
- **作者(Author)**: The Tauri Programme
- **许可协议(License)**: [MIT](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/updater/LICENSE_MIT) OR [MIT](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/updater/LICENSE_MIT)/[Apache 2.0](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/updater/LICENSE_APACHE-2.0)
- **用途(Usage)**: 实现检查更新功能 (Implement updater for Dioxus apps)
- **版权声明(Copyright)**:
  ```
  Copyright (c) 2015 - Present - The Tauri Programme within The Commons Conservancy.
  ```
- **主要修改(Key Modifications)**:
  - 从 Tauri 插件适配为通用 Rust 库 (Adapt for universal Rust crate)
  - 移除 Tauri 特定的运行时集成 (Remove Tauri-specific runtime integration)
  - 使用 `octocrab` 库进行 GitHub API 交互 (Use `octocrab` library for GitHub API interaction)
  - 移除 Linux 支持和其对应的依赖 (Remove Linux support and its corresponding deps)


---

> **详细信息**: 完整的归属信息请参阅 [**NOTICE**](./NOTICE) 文件
> **Detailed Info**: For complete attribution information, please refer to the [**NOTICE**](./NOTICE) file

## 许可协议

本项目采用双重许可协议，您可以选择其中任意一种：

* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) 或 https://www.apache.org/licenses/LICENSE-2.0)
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) 或 https://opensource.org/licenses/MIT)

### 贡献声明
除非您明确声明，否则根据 Apache-2.0 许可协议的定义，您有意提交的任何贡献都将按照上述双重许可协议进行许可，不附加任何额外条款或条件。
