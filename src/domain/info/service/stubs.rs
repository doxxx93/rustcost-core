// Temporary stub implementations for services that need migration
// TODO: Implement full service logic with new kube-rs client

use anyhow::{Result, anyhow};

// Stub for any service that hasn't been migrated yet
pub async fn not_implemented<T: Default>() -> Result<T> {
    Ok(T::default())
}

pub fn not_implemented_sync<T: Default>() -> Result<T> {
    Ok(T::default())
}
