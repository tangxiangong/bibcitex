<div align=center>
<img src="assets/transparent_logo.png" width="120" alt="BibCiTeX">
<h1>BibCiTeX</h1>
<p align="center">
跨平台 BibTeX 文献快捷引用工具
</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![Dioxus](https://img.shields.io/badge/dioxus-0.6-green.svg)](https://dioxuslabs.com)

</div>

## 简介

BibCiTeX 是一个使用 Rust 和 Dioxus 框架开发的跨平台 BibTeX 文献管理与快捷引用工具。
## ✨ 主要特性

### 🔍 智能搜索
- 快速全文搜索 BibTeX 条目
- 支持作者、标题、关键词等多字段搜索
- 实时搜索结果展示

### ⌨️ 全局快捷键
- **Cmd/Ctrl+K** 快速打开 Spotlight 风格的搜索窗口
- 无需切换应用，随时随地快速查找引用 (TODO)

### 📋 便捷引用
- 一键复制引用键 (cite key) 到剪贴板
- 支持多种引用格式
- 直接插入到当前编辑器光标位置 (TODO)

### 📚 文献管理
- 导入和解析 BibTeX 文件
- 文献详情查看
- 支持多个文献库管理

### 🎨 现代化界面
- 基于 TailwindCSS 和 DaisyUI 的现代化设计
- 响应式布局，适配不同屏幕尺寸
- 直观的用户交互体验

## 🏗️ 技术架构

### 核心技术栈
- **前端框架**: Dioxus (类 React 的 Rust 框架)
- **样式系统**: TailwindCSS + DaisyUI
- **BibTeX 解析**: `biblatex` crate
- **跨平台桌面原生集成**: 全局快捷键、文件对话框、剪贴板访问
- **构建工具**: Just (任务自动化)

### 项目结构
```
bibcitex/
├── src/                    # 主应用代码
│   ├── components/         # UI 组件
│   │   ├── bibliography.rs # 文献库组件
│   │   ├── reference.rs    # 引用条目组件
│   │   ├── helper.rs       # 助手组件
│   │   └── math.rs         # 数学公式渲染
│   ├── views/              # 应用视图
│   │   ├── home.rs         # 主页视图
│   │   ├── references.rs   # 引用管理视图
│   │   ├── helper.rs       # 助手视图
│   │   └── nav.rs          # 导航组件
│   └── lib.rs              # 应用入口和全局状态
├── bibcitex-core/          # 核心库
│   └── src/
│       ├── bib.rs          # BibTeX 解析
│       ├── search.rs       # 搜索功能
│       ├── setting.rs      # 设置管理
│       ├── msc.rs          # MSC 查询
│       └── utils.rs        # 工具函数
├── assets/                 # 静态资源
│   ├── icons/              # 图标文件
│   └── tailwind.css        # 编译后的样式
└── data/                   # 用户数据目录
```

## 🚀 快速开始

### 环境要求
- Rust 2024 edition 或更高版本
- Bun (用于 CSS 编译)
- 操作系统: macOS, Windows, Linux
- WebView

### 安装依赖
```bash
# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Bun (用于 TailwindCSS)
curl -fsSL https://bun.sh/install | bash

# 安装 Just (任务运行器)
cargo install just

# 克隆项目
git clone https://github.com/tangxiangong/bibcitex.git
cd bibcitex

# 安装前端依赖
bun install
```

### 开发运行
```bash
# 启动开发服务器 (并行运行 CSS 监听和 Dioxus 服务器)
just serve

# 或者分别运行
just dx-serve       # 启动 Dioxus 开发服务器
just css-watch      # 监听并编译 TailwindCSS
```

### 构建发布版本
```bash
# 编译并压缩 CSS
just css-minify

# 构建发布版本
cargo build --release
```

### 其他有用命令
```bash
# 格式化代码
just fmt

# 编译 CSS (一次性)
just css

# 运行测试
cargo test

# 运行 Clippy 检查
cargo clippy
```

## 📖 使用指南

### 导入文献库
1. 启动应用
2. 点击"添加文献库"按钮
3. 选择您的 BibTeX 文件 (.bib)
4. 应用将自动解析并加载文献条目

### 快速搜索引用
1. 在任何应用中按 **Cmd/Ctrl+K**
2. 输入关键词搜索文献
3. 选择目标文献条目
4. 按 Enter 复制引用键到剪贴板

### 查看文献详情
1. 在引用列表中点击任意条目
2. 右侧抽屉将显示详细信息
3. 可查看完整的 BibTeX 条目内容

## 🛣️ 开发路线图

### 🔄 进行中
- [ ] Spotlight 风格的全局搜索助手
- [ ] macOS 系统级无焦点窗口实现
- [ ] 快捷键自定义设置
- [ ] 多种引用格式支持 (`\cite`, `\citep`, `\citet` 等)

### 📋 计划中
- [ ] 文献库删除功能
- [ ] 完整的搜索功能优化
- [ ] 文献分类和标签系统
- [ ] 导出和同步功能

### 🎨 UI/UX 改进
- [ ] 完整的 UI 设计系统
- [ ] 自定义主题支持
- [ ] 更好的响应式设计

## 🤝 贡献指南

我们欢迎各种形式的贡献！请查看 [DEVELOPMENT.md](DEVELOPMENT.md) 了解开发环境设置。

### 贡献方式
- 🐛 报告 Bug
- 💡 提出新功能建议
- 📖 改进文档
- 🔧 提交代码

### 开发流程
1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 运行测试和格式化: `just fmt && cargo test`
5. 提交 Pull Request

## 📄 许可协议

本项目采用双重许可协议，您可以选择其中任意一种：

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) 或 https://www.apache.org/licenses/LICENSE-2.0)
* MIT License ([LICENSE-MIT](LICENSE-MIT) 或 https://opensource.org/licenses/MIT)

### 贡献声明
除非您明确声明，否则根据 Apache-2.0 许可协议的定义，您有意提交的任何贡献都将按照上述双重许可协议进行许可，不附加任何额外条款或条件。
