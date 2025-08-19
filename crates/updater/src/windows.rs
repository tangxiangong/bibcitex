use crate::{Result, Updater};
use std::{ffi::OsStr, path::PathBuf};

type WindowsUpdaterType = (PathBuf, Option<tempfile::TempPath>);

const NSIS_ARGS: &[&str] = &["/P", "/R"];

impl Updater {
    pub(crate) fn install_inner(&self, bytes: &[u8]) -> Result<()> {
        use std::iter::once;
        use windows_sys::{
            Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOW},
            w,
        };

        let updater_type = self.extract_exe(bytes)?;

        let current_args = &self.current_exe_args()[1..];
        let nsis_args = current_args
            .iter()
            .map(escape_nsis_current_exe_arg)
            .collect::<Vec<_>>();

        let installer_args: Vec<&OsStr> = NSIS_ARGS
            .iter()
            .map(OsStr::new)
            .chain(once(OsStr::new("/UPDATE")))
            .chain(once(OsStr::new("/ARGS")))
            .chain(nsis_args.iter().map(OsStr::new))
            .chain(self.installer_args())
            .collect();

        let file = updater_type.0.as_os_str().to_os_string();
        let file = encode_wide(file);

        let parameters = installer_args.join(OsStr::new(" "));
        let parameters = encode_wide(parameters);

        unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                w!("open"),
                file.as_ptr(),
                parameters.as_ptr(),
                std::ptr::null(),
                SW_SHOW,
            )
        };

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

    fn make_temp_dir(&self) -> Result<PathBuf> {
        Ok(tempfile::Builder::new()
            .prefix(&format!(
                "{}-{}-updater-",
                self.app_name,
                self.latest_version()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ))
            .tempdir()?
            .keep())
    }

    fn extract_exe(&self, bytes: &[u8]) -> Result<WindowsUpdaterType> {
        let (path, temp) = self.write_to_temp(bytes, ".exe")?;
        Ok((path, temp))
    }

    fn write_to_temp(
        &self,
        bytes: &[u8],
        ext: &str,
    ) -> Result<(PathBuf, Option<tempfile::TempPath>)> {
        use std::io::Write;

        let temp_dir = self.make_temp_dir()?;
        let mut temp_file = tempfile::Builder::new()
            .prefix(&format!(
                "{}-{}-installer",
                self.app_name,
                self.latest_version()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ))
            .suffix(ext)
            .rand_bytes(0)
            .tempfile_in(temp_dir)?;
        temp_file.write_all(bytes)?;

        let temp = temp_file.into_temp_path();
        Ok((temp.to_path_buf(), Some(temp)))
    }
}

fn encode_wide(string: impl AsRef<OsStr>) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    string
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

// adapted from https://github.com/rust-lang/rust/blob/1c047506f94cd2d05228eb992b0a6bbed1942349/library/std/src/sys/args/windows.rs#L174
fn escape_nsis_current_exe_arg(arg: &&OsStr) -> String {
    let arg = arg.to_string_lossy();
    let mut cmd: Vec<char> = Vec::new();

    // compared to std we additionally escape `/` so that nsis won't interpret them as a beginning of an nsis argument.
    let quote = arg.chars().any(|c| c == ' ' || c == '\t' || c == '/') || arg.is_empty();
    let escape = true;
    if quote {
        cmd.push('"');
    }
    let mut backslashes: usize = 0;
    for x in arg.chars() {
        if escape {
            if x == '\\' {
                backslashes += 1;
            } else {
                if x == '"' {
                    // Add n+1 backslashes to total 2n+1 before internal '"'.
                    cmd.extend((0..=backslashes).map(|_| '\\'));
                }
                backslashes = 0;
            }
        }
        cmd.push(x);
    }
    if quote {
        // Add n backslashes to total 2n before ending '"'.
        cmd.extend((0..backslashes).map(|_| '\\'));
        cmd.push('"');
    }
    cmd.into_iter().collect()
}
