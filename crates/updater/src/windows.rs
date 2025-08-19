// Copyright (c) 2025 BibCiTeX Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// This file contains code derived from tauri-plugin-updater
// Original source: https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/updater
// Copyright (c) 2015 - Present - The Tauri Programme within The Commons Conservancy.
// Licensed under MIT OR MIT/Apache-2.0

use crate::{Result, Updater};
use std::{ffi::OsStr, path::PathBuf, thread, time::Duration};
use windows_sys::{
    Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOW},
    w,
};

type WindowsUpdaterType = (PathBuf, Option<tempfile::TempPath>);

impl Updater {
    pub(crate) fn install_inner(&self, bytes: &[u8]) -> Result<()> {
        let updater_type = self.extract_exe(bytes)?;

        // Verify the installer file exists and is executable
        if !updater_type.0.exists() {
            return Err(crate::Error::InvalidUpdaterFormat);
        }

        let file = updater_type.0.as_os_str().to_os_string();
        let file = encode_wide(file);

        // Open the installer for manual installation with admin privileges if needed
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                w!("runas"), // Request administrator privileges for installation
                file.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null(),
                SW_SHOW,
            )
        } as i32;

        // Check the result of ShellExecuteW
        // Values <= 32 indicate an error
        if result <= 32 {
            return match result {
                5 => Err(crate::Error::InsufficientPrivileges), // ERROR_ACCESS_DENIED
                32 => Err(crate::Error::FileInUse),             // ERROR_SHARING_VIOLATION
                1223 => Err(crate::Error::UserCancelledElevation), // ERROR_CANCELLED (UAC cancelled)
                _ => Err(crate::Error::InstallerExecutionFailed(result)),
            };
        }

        // Give the installer a moment to start before exiting
        thread::sleep(Duration::from_millis(500));

        std::process::exit(0);
    }

    fn make_temp_dir(&self) -> Result<PathBuf> {
        // Try to create temp directory in system temp first, fallback to current directory
        let temp_dir = tempfile::Builder::new()
            .prefix(&format!(
                "{}-{}-updater-",
                self.app_name,
                self.latest_version()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ))
            .tempdir();

        match temp_dir {
            Ok(dir) => {
                let path = dir.keep();
                // Ensure the directory exists and is writable
                if path.exists() && path.is_dir() {
                    Ok(path)
                } else {
                    Err(crate::Error::TempDirNotFound)
                }
            }
            Err(_) => {
                // Fallback: try to create in current directory
                let fallback_dir = std::env::current_dir()?.join(format!(
                    "{}-{}-updater-temp",
                    self.app_name,
                    self.latest_version()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                ));

                std::fs::create_dir_all(&fallback_dir)?;
                Ok(fallback_dir)
            }
        }
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
            .tempfile_in(&temp_dir)?;

        temp_file.write_all(bytes)?;
        temp_file.flush()?; // Ensure all data is written to disk

        let temp = temp_file.into_temp_path();
        let temp_path = temp.to_path_buf();

        // Verify the file was written correctly
        if !temp_path.exists() || std::fs::metadata(&temp_path)?.len() != bytes.len() as u64 {
            return Err(crate::Error::InvalidUpdaterFormat);
        }

        Ok((temp_path, Some(temp)))
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
