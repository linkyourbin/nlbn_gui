use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::ProgressUpdate;

/// Application global state
#[derive(Default)]
pub struct AppState {
    /// Current conversion progress
    pub current_progress: Arc<RwLock<Option<ProgressUpdate>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_progress: Arc::new(RwLock::new(None)),
        }
    }
}
