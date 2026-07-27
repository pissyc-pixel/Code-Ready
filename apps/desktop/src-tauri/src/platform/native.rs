use std::fmt::{Display, Formatter};

use crate::domain::contracts::PlatformId;

use super::PlatformAdapter;

#[derive(Debug)]
pub struct UnsupportedPlatformError;

impl Display for UnsupportedPlatformError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("unsupported Code-Ready platform")
    }
}

impl std::error::Error for UnsupportedPlatformError {}

#[derive(Debug)]
pub struct NativePlatformAdapter {
    platform: PlatformId,
}

impl NativePlatformAdapter {
    pub fn new() -> Result<Self, UnsupportedPlatformError> {
        Self::current()
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    fn current() -> Result<Self, UnsupportedPlatformError> {
        Ok(Self {
            platform: PlatformId::WindowsX64,
        })
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn current() -> Result<Self, UnsupportedPlatformError> {
        Ok(Self {
            platform: PlatformId::MacosArm64,
        })
    }

    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64")
    )))]
    fn current() -> Result<Self, UnsupportedPlatformError> {
        Err(UnsupportedPlatformError)
    }
}

impl PlatformAdapter for NativePlatformAdapter {
    fn platform(&self) -> PlatformId {
        self.platform.clone()
    }
}
