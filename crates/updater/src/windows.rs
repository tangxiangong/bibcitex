#[cfg(windows)]
enum WindowsUpdaterType {
    Nsis {
        path: PathBuf,
        #[allow(unused)]
        temp: Option<tempfile::TempPath>,
    },
    Msi {
        path: PathBuf,
        #[allow(unused)]
        temp: Option<tempfile::TempPath>,
    },
}

#[cfg(windows)]
impl WindowsUpdaterType {
    fn nsis(path: PathBuf, temp: Option<tempfile::TempPath>) -> Self {
        Self::Nsis { path, temp }
    }

    fn msi(path: PathBuf, temp: Option<tempfile::TempPath>) -> Self {
        Self::Msi {
            path: path.wrap_in_quotes(),
            temp,
        }
    }
}

/// Windows
#[cfg(windows)]
impl Update {
    /// ### Expected structure:
    /// ├── [AppName]-[version]-windows-x86_64.msi              # Application MSI (x86_64)
    /// ├── [AppName]-[version]-windows-arm64.msi               # Application MSI (arm64)
    /// ├── [AppName]-[version]-windows-x86_64.exe              # NSIS installer (x86_64)
    /// ├── [AppName]-[version]-windows-arm64.exe               # NSIS installer (arm64)
    fn install_inner(&self, bytes: &[u8]) -> Result<()> {
        use std::iter::once;
        use windows_sys::{
            Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOW},
            w,
        };

        let updater_type = self.extract(bytes)?;

        let install_mode = crate::config::WindowsUpdateInstallMode::default();
        let current_args = &self.current_exe_args()[1..];
        let msi_args;
        let nsis_args;

        let installer_args: Vec<&OsStr> = match &updater_type {
            WindowsUpdaterType::Nsis { .. } => {
                nsis_args = current_args
                    .iter()
                    .map(escape_nsis_current_exe_arg)
                    .collect::<Vec<_>>();

                install_mode
                    .nsis_args()
                    .iter()
                    .map(OsStr::new)
                    .chain(once(OsStr::new("/UPDATE")))
                    .chain(once(OsStr::new("/RESTART")))
                    .chain(once(OsStr::new("/CLOSEAPPLICATIONS"))) // 强制关闭应用
                    .chain(once(OsStr::new("/RESTARTAPPLICATIONS"))) // 重启后恢复
                    .chain(once(OsStr::new("/ARGS")))
                    .chain(nsis_args.iter().map(OsStr::new))
                    .chain(self.installer_args())
                    .collect()
            }
            WindowsUpdaterType::Msi { path, .. } => {
                let escaped_args = current_args
                    .iter()
                    .map(escape_msi_property_arg)
                    .collect::<Vec<_>>()
                    .join(" ");
                msi_args = OsString::from(format!("LAUNCHAPPARGS=\"{escaped_args}\""));

                [OsStr::new("/i"), path.as_os_str()]
                    .into_iter()
                    .chain(install_mode.msiexec_args().iter().map(OsStr::new))
                    .chain(once(OsStr::new("/forcerestart"))) // 强制重启
                    .chain(self.installer_args())
                    .chain(once(OsStr::new("AUTOLAUNCHAPP=True")))
                    .chain(once(msi_args.as_os_str()))
                    .collect()
            }
        };

        let file = match &updater_type {
            WindowsUpdaterType::Nsis { path, .. } => path.as_os_str().to_os_string(),
            WindowsUpdaterType::Msi { .. } => std::env::var("SYSTEMROOT").as_ref().map_or_else(
                |_| OsString::from("msiexec.exe"),
                |p| OsString::from(format!("{p}\\System32\\msiexec.exe")),
            ),
        };
        let file = encode_wide(file);

        // 正确处理包含空格的参数，避免参数解析错误
        let parameters = installer_args
            .iter()
            .map(|arg| {
                let arg_str = arg.to_string_lossy();
                if arg_str.contains(' ') && !arg_str.starts_with('"') {
                    format!("\"{}\"", arg_str)
                } else {
                    arg_str.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        let parameters = encode_wide(parameters);

        // 使用 runas 动词请求管理员权限
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                w!("runas"), // 请求管理员权限
                file.as_ptr(),
                parameters.as_ptr(),
                std::ptr::null(),
                SW_SHOW,
            )
        };

        // 检查启动结果并返回具体错误
        if result as i32 <= 32 {
            return match result as i32 {
                5 => Err(Error::InsufficientPrivileges), // ERROR_ACCESS_DENIED
                1223 => Err(Error::UserCancelledElevation), // ERROR_CANCELLED (用户取消UAC)
                32 => Err(Error::FileInUse),             // ERROR_SHARING_VIOLATION
                2 => Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Installer file not found",
                ))),
                _ => Err(Error::InstallerExecutionFailed(result as i32)),
            };
        }

        // 延迟退出，给安装程序更多时间启动
        std::thread::sleep(std::time::Duration::from_millis(2000));
        std::process::exit(0);
    }

    fn installer_args(&self) -> Vec<&OsStr> {
        self.installer_args
            .iter()
            .map(OsStr::new)
            .collect::<Vec<_>>()
    }

    fn current_exe_args(&self) -> Vec<&OsStr> {
        self.current_exe_args
            .iter()
            .map(OsStr::new)
            .collect::<Vec<_>>()
    }

    fn extract(&self, bytes: &[u8]) -> Result<WindowsUpdaterType> {
        if infer::archive::is_zip(bytes) {
            return self.extract_zip(bytes);
        }

        self.extract_exe(bytes)
    }

    fn make_temp_dir(&self) -> Result<PathBuf> {
        Ok(tempfile::Builder::new()
            .prefix(&format!("{}-{}-updater-", self.app_name, self.version))
            .tempdir()?
            .keep())
    }

    fn extract_zip(&self, bytes: &[u8]) -> Result<WindowsUpdaterType> {
        let temp_dir = self.make_temp_dir()?;

        let archive = Cursor::new(bytes);
        let mut extractor = zip::ZipArchive::new(archive)?;
        extractor.extract(&temp_dir)?;

        let paths = std::fs::read_dir(&temp_dir)?;
        for path in paths {
            let path = path?.path();
            let ext = path.extension();
            if ext == Some(OsStr::new("exe")) {
                return Ok(WindowsUpdaterType::nsis(path, None));
            } else if ext == Some(OsStr::new("msi")) {
                return Ok(WindowsUpdaterType::msi(path, None));
            }
        }

        Err(crate::Error::BinaryNotFoundInArchive)
    }

    fn extract_exe(&self, bytes: &[u8]) -> Result<WindowsUpdaterType> {
        if infer::app::is_exe(bytes) {
            let (path, temp) = self.write_to_temp(bytes, ".exe")?;
            Ok(WindowsUpdaterType::nsis(path, temp))
        } else if infer::archive::is_msi(bytes) {
            let (path, temp) = self.write_to_temp(bytes, ".msi")?;
            Ok(WindowsUpdaterType::msi(path, temp))
        } else {
            Err(crate::Error::InvalidUpdaterFormat)
        }
    }

    fn write_to_temp(
        &self,
        bytes: &[u8],
        ext: &str,
    ) -> Result<(PathBuf, Option<tempfile::TempPath>)> {
        use std::io::Write;

        let temp_dir = self.make_temp_dir()?;
        let mut temp_file = tempfile::Builder::new()
            .prefix(&format!("{}-{}-installer", self.app_name, self.version))
            .suffix(ext)
            .rand_bytes(0)
            .tempfile_in(temp_dir)?;
        temp_file.write_all(bytes)?;

        let temp = temp_file.into_temp_path();
        Ok((temp.to_path_buf(), Some(temp)))
    }
}
