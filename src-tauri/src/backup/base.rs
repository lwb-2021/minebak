use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum MinecraftInstanceType {
    Normal = 0,
    VersionIsolated = 1,
    MultiMC = 2,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftInstanceMetadata {}

impl MinecraftInstanceMetadata {
    pub fn parse(path: PathBuf, instance_type: MinecraftInstanceType) -> Result<Self> {
        // TODO
        Ok(MinecraftInstanceMetadata {})
    }
}
