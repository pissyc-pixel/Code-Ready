use crate::domain::contracts::PlatformId;

use super::PlatformAdapter;

#[derive(Clone, Debug)]
pub struct FakePlatformAdapter {
    platform: PlatformId,
}

impl FakePlatformAdapter {
    pub fn new(platform: PlatformId) -> Self {
        Self { platform }
    }
}

impl PlatformAdapter for FakePlatformAdapter {
    fn platform(&self) -> PlatformId {
        self.platform.clone()
    }
}
