# BibCiTeX for Windows

C# / WinUI 3（Windows App SDK 2.5.1 稳定版）主窗口与 Spotlight helper；共享 Rust Core、XPaste、原生 Helper 状态。主窗使用系统 Mica（不支持时使用 Desktop Acrylic），helper 使用系统 Desktop Acrylic。快捷键 `Win+Shift+K`，空闲搜索栏高 56 DIP。公式由 CSharpMath + SkiaSharp 在原生控件内渲染，无 WebView。

最低运行系统为 Windows 10 1809（内部版本 17763），支持 x64 和 ARM64。编译使用的 Windows SDK 版本与最低运行版本分别配置。

## Visual Studio

安装 Visual Studio 2026 的 Windows 应用开发、C++ 桌面开发工具和对应 Windows SDK，.NET 10 LTS、Rust MSVC 工具链。首次选择架构时，有 rustup 则自动补齐所选目标和生成器宿主目标。直接打开 `windows/BibCiTeX.sln`，选择 `Debug` / `Release` 和 `x64` / `ARM64`，执行“生成解决方案”即可。

MSBuild 会自动构建 Rust DLL、运行 Interoptopus 生成器，再编译 WinUI；干净检出不需要预先执行脚本、Just 或手工生成绑定。Rust 源码变化会触发构建检查，设计时 IntelliSense 不启动 Cargo。找不到 Cargo 时检查 PATH，再检查 `%USERPROFILE%/.cargo/bin/cargo.exe`，缺失时输出明确错误。工具链沿用当前 rustup / `RUSTUP_TOOLCHAIN`，不固定版本；没有 rustup 则使用已安装目标。显式设置 `CargoExecutable` 时不会自动改用其他工具链，除非同时指定匹配的 `RustupExecutable`。

项目使用普通 `Project` 启动项，Visual Studio F5 运行非打包 WinUI 程序，不依赖包身份。运行由用户操作；本次没有启动应用或 GUI。macOS 已验证共享动态库和 C# 语义，完整 Windows XAML 编译及部署须在 Windows 验证。

CLI 和 IDE 使用同一构建链：

```powershell
dotnet build windows/BibCiTeX.sln -c Debug -p:Platform=x64
dotnet build windows/BibCiTeX.sln -c Release -p:Platform=ARM64
```

`windows/scripts/build.ps1` 只是上述解决方案构建的参数入口，不重复构建 Rust。生成的绑定位于 `bindings/generated/csharp`，Rust DLL 位于 `target/<目标三元组>/<debug或release>/bibcitex_csharp.dll`。

## 原生更新发布

Windows uses unpackaged, framework-dependent WinUI 3 with Velopack 1.2.161. EXE and MSI installers detect and install the matching .NET 10 runtime and Visual C++ runtime when missing. Before starting WinUI on the first application launch, `WindowsRuntime` checks Windows App Runtime 2.5.1, downloads the matching Microsoft installer if needed, verifies its pinned SHA-256, installs it and retries initialization. Existing compatible installations are reused; missing dependencies require internet access. Silent installation defers Windows App Runtime provisioning until the first application launch. `Program.Main` handles Velopack hooks before this bootstrap, so packaging and uninstall hooks do not download runtimes. When upgrading the Windows App SDK runtime package, update the bootstrap version, URLs and verified hashes together. Settings are stored outside the replaceable application folder. EXE and MSI share the Velopack update layout; the updater supports download progress, cancellation, size/hash verification and restart. Automatic mode downloads in the background and applies on the next launch.

```powershell
./windows/scripts/publish.ps1 -Architecture x64 -Version 0.7.0 -Channel stable -BuildNumber 1 -OutputDirectory dist/release/windows-x64
```

Use an empty output directory. This generates EXE, MSI, a full `.nupkg`, and `releases.win-x64-stable.json`. ARM64 uses `-Architecture ARM64`. Release builds require reviewed `release-notes/<version>/zh-Hans.md` and `en.md`. The workflow uploads all artifacts to GitHub Releases and advances `update-feed` channel pointers only after verification. It does not require a Windows code-signing certificate or Microsoft Store. Unsigned installers can still trigger OS warnings.

`BibCiTeX.exe --prepare-runtime` provisions Windows App Runtime without opening the UI and returns a nonzero exit code on failure. Only a missing compatible framework triggers a download; other bootstrap failures are reported without reinstalling the runtime.

Channel preferences are stable/beta/alpha, independent of x64/arm64. Selecting stable does not downgrade an installed preview. Markdown notes use the application's selected language and native WinUI text controls. See [the release protocol](../docs/updates.md).

## 资源

UI 图标均加载独立 SVG 文件；浅色资源链接 `public/icons/windows`，深色资源位于 `BibCiTeX/Assets/Icons/Dark`，均为 Microsoft Fluent SVG 的衍生资源。许可证随程序复制到 `Assets/Icons/LICENSE`。应用和系统托盘标识复用现有 `assets/app-icons`。

## 测试

先生成绑定并构建对应主机架构的 `bibcitex-csharp` 动态库，再运行：

```powershell
dotnet run --project windows/Tests/BibCiTeX.IntegrationTests.csproj -c Release -- $RustLibraryPath
```

该集成测试只访问临时 BibTeX 文件，验证 Unicode、完整字段、数学块、错误、并发、Wire 内存所有权以及文件缓存更新，不调用文献库配置、剪贴板、粘贴或界面。

本地化使用由原生控件持有的 XAML Binding，避免 C# 控件包装对象被 GC 回收后，菜单和既有列表退出语言刷新。`Strings/en-US` 与 `Strings/zh-CN` 声明原生资源语言，界面词典仍共用根目录 JSON。

不编写或运行 UI/GUI 测试；界面通过人工验收，CI 保留实际应用编译和非 UI 集成测试。
