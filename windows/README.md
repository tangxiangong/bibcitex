# BibCiTeX for Windows

C# / WinUI 3（Windows App SDK 2.5.1 稳定版）主窗口与 Spotlight helper；共享 Rust Core、XPaste、原生 Helper 状态。主窗使用系统 Mica（不支持时使用 Desktop Acrylic），helper 使用系统 Desktop Acrylic。快捷键 `Win+Shift+K`，空闲搜索栏高 56 DIP。公式由 CSharpMath + SkiaSharp 在原生控件内渲染，无 WebView。

## Visual Studio

安装 Visual Studio 2026 的 Windows 应用开发、C++ 桌面开发工具和对应 Windows SDK，.NET 10 LTS、Rust MSVC 工具链。首次选择架构时，有 rustup 则自动补齐所选目标和生成器宿主目标。直接打开 `windows/BibCiTeX.sln`，选择 `Debug` / `Release` 和 `x64` / `ARM64`，执行“生成解决方案”即可。

MSBuild 会自动构建 Rust DLL、运行 Interoptopus 生成器，再编译 WinUI；干净检出不需要预先执行脚本、Just 或手工生成绑定。Rust 源码变化会触发构建检查，设计时 IntelliSense 不启动 Cargo。找不到 Cargo 时检查 PATH，再检查 `%USERPROFILE%/.cargo/bin/cargo.exe`，缺失时输出明确错误。工具链沿用当前 rustup / `RUSTUP_TOOLCHAIN`，不固定版本；没有 rustup 则使用已安装目标。显式设置 `CargoExecutable` 时不会自动改用其他工具链，除非同时指定匹配的 `RustupExecutable`。

项目配置了 `BibCiTeX (Package)` 启动项，Visual Studio F5 使用 MSIX 包身份。运行由用户操作；本次没有启动应用或 GUI。macOS 已验证共享动态库和 C# 语义，完整 Windows XAML 编译及部署须在 Windows 验证。

CLI 和 IDE 使用同一构建链：

```powershell
dotnet build windows/BibCiTeX.sln -c Debug -p:Platform=x64
dotnet build windows/BibCiTeX.sln -c Release -p:Platform=ARM64
```

`windows/scripts/build.ps1` 只是上述解决方案构建的参数入口，不重复构建 Rust。生成的绑定位于 `bindings/generated/csharp`，Rust DLL 位于 `target/<目标三元组>/<debug或release>/bibcitex_csharp.dll`。

## 原生更新发布

使用独立且空的输出目录；每次发布递增四段版本号。证书必须已安装在当前用户证书库，具有私钥，Subject 必须与 Publisher 完全相同。通过 `WINDOWS_TIMESTAMP_URL` 或 `-TimestampServer` 提供证书颁发机构的时间戳地址。开发默认 `CN=BibCiTeX` 不代表已经签名或具有受信任的证书。

```powershell
./windows/scripts/publish.ps1 -Architecture x64 -Version $ReleaseVersion `
  -Publisher $CertificateSubject -CertificateThumbprint $Thumbprint `
  -PublishBaseUri $ReleaseHttpsUri -OutputDirectory $ReleaseDirectory
```

脚本生成签名 MSIX 和 `.appinstaller`，不上传、不安装、不启动。将二者发布到指定的真实 HTTPS 地址，再通过 `.appinstaller` 安装；仅侧载 `.msix` 没有 App Installer 更新源。ARM64 使用单独架构的 `.appinstaller`。

“检查更新”使用 Windows `Package.CheckUpdateAvailabilityAsync` 和 `PackageManager.RequestAddPackageByAppInstallerFileAsync`；安装确认、签名校验和替换由系统部署服务完成。未配置更新源时显示实际错误，不编造发行 URL，不自行下载替换 EXE。

- [Microsoft: update non-Store apps](https://learn.microsoft.com/en-us/windows/msix/non-store-developer-updates)
- [Microsoft: App Installer install/update API](https://learn.microsoft.com/en-us/uwp/api/windows.management.deployment.packagemanager.requestaddpackagebyappinstallerfileasync)
- [CSharpMath](https://github.com/verybadcat/CSharpMath)

## 资源

UI 图标均加载独立 SVG 文件；浅色资源链接 `public/icons/windows`，深色资源位于 `BibCiTeX/Assets/Icons/Dark`，均为 Microsoft Fluent SVG 的衍生资源。许可证随程序复制到 `Assets/Icons/LICENSE`。应用和系统托盘标识复用现有 `assets/app-icons`。

## 测试

先生成绑定并构建对应主机架构的 `bibcitex-csharp` 动态库，再运行：

```powershell
dotnet run --project windows/Tests/BibCiTeX.IntegrationTests.csproj -c Release -- $RustLibraryPath
```

该集成测试只访问临时 BibTeX 文件，验证 Unicode、完整字段、数学块、错误、并发、Wire 内存所有权以及文件缓存更新；另外直接在内存调用 Skia 渲染分数、积分、矩阵、希腊字母和长公式，检查 DPI、颜色和边界像素，不调用文献库配置、剪贴板、粘贴或界面，不保存图片。

`windows/Tests/UiSemantics/BibCiTeX.UiSemantics.csproj` 是不可运行的 C# 语义检查项目，引用真实 WinUI 程序集并编译全部界面 C# 源文件。它明确替换 Windows XAML 编译器生成的 `InitializeComponent` 边界为抛异常声明，不生成 XAML 资源，也不证明应用构建通过；Windows CI 仍须构建真正的 `BibCiTeX.csproj`。
