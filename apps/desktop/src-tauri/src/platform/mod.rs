use crate::domain::contracts::PlatformId;

pub mod fake;
pub mod native;

pub trait PlatformAdapter: Send + Sync {
    fn platform(&self) -> PlatformId;
}
