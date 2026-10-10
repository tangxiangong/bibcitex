<div align="center">
  <img src="public/transparent_logo.png" width="120" alt="BibCiTeX">

  <p>English | <a href="README-zh.md">简体中文</a></p>

  <p>
    A quick citation tool for BibTeX bibliographies
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
    <strong>Quick Downloads</strong>
  </p>

  <p>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.7.3/BibCiTeX-0.7.3-macos-arm64.dmg">
      <img src="https://img.shields.io/badge/macOS-Apple Silicon-000000?style=for-the-badge&logo=apple&logoColor=white" alt="macOS Apple Silicon">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.7.3/BibCiTeX-0.7.3-macos-x86_64.dmg">
      <img src="https://img.shields.io/badge/macOS-Intel-000000?style=for-the-badge&logo=apple&logoColor=white" alt="macOS Intel">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.7.3/BibCiTeX-0.7.3-windows-arm64.exe">
      <img src="https://img.shields.io/badge/Windows-ARM64-0078D4?style=for-the-badge&logo=windows&logoColor=white" alt="Windows ARM64">
    </a>
    <a href="https://github.com/tangxiangong/bibcitex/releases/download/v0.7.3/BibCiTeX-0.7.3-windows-x64.exe">
      <img src="https://img.shields.io/badge/Windows-x86__64-0078D4?style=for-the-badge&logo=windows&logoColor=white" alt="Windows x86_64">
    </a>
  </p>
</div>

## Introduction

**BibCiTeX** is a quick citation tool for **BibTeX** bibliographies, powered by a shared **Rust** core.

It supports macOS and Windows, with bibliography search, citation copying, and cross-application pasting.

What's new in v0.7.3: [English release notes](release-notes/0.7.3/en.md) · [Chinese release notes](release-notes/0.7.3/zh-Hans.md).

### Core Features

- **One-click copying**: Copy citations with ease.
- **Cross-application pasting**: Paste citations directly into your workflow.

## Installation

### Supported Systems

| Platform | Supported OS Versions | Architectures |
| --- | --- | --- |
| macOS | macOS 13 (Ventura) or later | Apple Silicon (ARM64), Intel (x86_64) |
| Windows | Windows 10 version 1809 (OS build 17763) or later, including Windows 11 | x64, ARM64 |

### Downloads

Download the latest installer for your platform and architecture from the [**Releases page**](https://github.com/tangxiangong/bibcitex/releases).

### macOS Installation

You can also install through the project's [Homebrew tap](https://github.com/tangxiangong/homebrew-tap), which supports Apple Silicon and Intel Macs running macOS 13 or later:

```bash
brew install --cask tangxiangong/tap/bibcitex
```

The app supports built-in updates. To update through Homebrew, run `brew update`, followed by `brew upgrade --cask --greedy bibcitex`.

If macOS reports that `BibCiTeX` is damaged, open Terminal and run:

```bash
sudo xattr -dr com.apple.quarantine /Applications/BibCiTeX.app
```

> **Tip**: This message is caused by macOS security restrictions. The command above removes the app's quarantine attribute so you can open it.

## Interface Preview

The animations below show earlier versions of the app.

<div align="center">

### Core Features

|                          Add `.bib` Files                          |                              Bibliography List                               |                           Smart Search                           |
| :----------------------------------------------------------------: | :--------------------------------------------------------------------------: | :--------------------------------------------------------------: |
| [<img src="assets/add_bib.gif" width="120">](./assets/add_bib.gif) | [<img src="assets/show_details.gif" width="120">](./assets/show_details.gif) | [<img src="assets/search.gif" width="120">](./assets/search.gif) |
|                   *Quickly import BibTeX files*                    |                           *View reference details*                           |                   *Filter results as you type*                   |

|                         Details Sidebar                          |                       External Links                       |                        Copy Citations                        |
| :--------------------------------------------------------------: | :--------------------------------------------------------: | :----------------------------------------------------------: |
| [<img src="assets/drawer.gif" width="120">](./assets/drawer.gif) | [<img src="assets/url.gif" width="120">](./assets/url.gif) | [<img src="assets/copy.gif" width="120">](./assets/copy.gif) |
|                  *View details in the sidebar*                   |            *Quickly access external resources*             |          *Copy formatted citations with one click*           |

### Featured Functionality

<div style="margin: 20px 0;">
  <h4>Cross-Application Pasting</h4>
  <a href="assets/cross_paste.gif">
    <img src="assets/cross_paste.gif" alt="Cross-application paste demo">
  </a>
  <p><em>Paste citations across applications for seamless integration into your workflow.</em></p>
</div>

</div>

## TODO

The following features are planned and have not yet been implemented. See [TODO.md](./TODO.md) for the detailed roadmap.

- [ ] **Search improvements**: Multiple keywords, fuzzy matching, relevance ranking, and performance optimization.
- [ ] **Advanced search**: Field queries, exact phrases, exclusions, and year ranges.
- [ ] **Tags**: Tag management, bulk addition and removal, and tag filtering.
- [ ] **Helper filter commands**: Type `@`, `$`, or `#` to show reference type, field, or tag suggestions, respectively, and use a compact input field to complete the filter.
- [ ] **Multiple selection**: Select multiple references and perform bulk operations in the main window and helper.
- [ ] **Copy and paste formats**: Bare citation keys, LaTeX and Typst citation presets, and combined citations for multiple references.

## Third-Party Code Attribution

### [crates/xpaste](./crates/xpaste)

- **Source**: [EcoPasteHub/EcoPaste](https://github.com/EcoPasteHub/EcoPaste)
- **Author**: EcoPasteHub
- **License**: [Apache 2.0](https://github.com/EcoPasteHub/EcoPaste/blob/master/LICENSE)
- **Usage**: Cross-application paste functionality.
- **Copyright**:
  ```
  Copyright (c) EcoPasteHub
  ```
- **Modifications**:
  - macOS: Replaced deprecated `objc` and `cocoa` APIs with `objc2` APIs.
  - Windows: Replaced deprecated `winapi` APIs with `windows-sys` APIs.

---

> **Details**: See [**NOTICE**](./NOTICE) for complete attribution information.

## License

This project is dual-licensed under either of the following, at your option:

* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

### Contributions

Unless you explicitly state otherwise, any contribution you intentionally submit for inclusion in this project, as defined in the Apache-2.0 license, will be dual-licensed as above, without any additional terms or conditions.
