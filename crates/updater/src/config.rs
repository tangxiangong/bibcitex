// This module is kept for potential future configuration needs
// Currently all configuration is handled directly in the updater

#[cfg(windows)]
#[derive(Debug, Clone)]
pub struct WindowsUpdateInstallMode {
    nsis_args: Vec<String>,
    msiexec_args: Vec<String>,
}

#[cfg(windows)]
impl Default for WindowsUpdateInstallMode {
    fn default() -> Self {
        Self {
            nsis_args: vec!["/S".to_string()],        // Silent install for NSIS
            msiexec_args: vec!["/quiet".to_string()], // Quiet install for MSI
        }
    }
}

#[cfg(windows)]
impl WindowsUpdateInstallMode {
    pub fn nsis_args(&self) -> &[String] {
        &self.nsis_args
    }

    pub fn msiexec_args(&self) -> &[String] {
        &self.msiexec_args
    }

    pub fn with_nsis_args(mut self, args: Vec<String>) -> Self {
        self.nsis_args = args;
        self
    }

    pub fn with_msiexec_args(mut self, args: Vec<String>) -> Self {
        self.msiexec_args = args;
        self
    }
}
