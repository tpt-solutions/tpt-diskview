use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupCategory {
    TempFiles,
    Duplicates,
    DockerVolumes,
    BrowserCache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupCandidate {
    pub path: String,
    pub size: u64,
    pub category: CleanupCategory,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub candidates: Vec<CleanupCandidate>,
    pub total_size: u64,
}
