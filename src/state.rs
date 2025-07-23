use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Serialize, Deserialize, Debug)]
struct Entry {
    id: String,
    timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct Manager {
    file_path: PathBuf,
    seen_ids: Arc<RwLock<HashMap<String, i64>>>,
    memory_duration: Duration,
}

impl Manager {
    pub fn new(file_name: impl AsRef<Path>, memory_duration: Duration) -> Self {
        Self {
            file_path: file_name.as_ref().to_path_buf(),
            seen_ids: Arc::new(RwLock::new(HashMap::new())),
            memory_duration,
        }
    }

    pub async fn is_seen(&self, id: &str) -> bool {
        let seen = self.seen_ids.read().await;
        seen.contains_key(id)
    }

    pub async fn add(&self, id: String, timestamp: i64) {
        let mut seen = self.seen_ids.write().await;
        seen.insert(id, timestamp);
    }

    pub async fn load(&self) -> anyhow::Result<()> {
        let path_str = self.file_path.to_string_lossy();
        match tokio::fs::read(&self.file_path).await {
            Ok(data) => {
                let entries: Vec<Entry> = serde_json::from_slice(&data)?;
                let cutoff = (SystemTime::now() - self.memory_duration)
                    .duration_since(UNIX_EPOCH)?
                    .as_secs() as i64;
                
                let mut seen_map = self.seen_ids.write().await;
                let mut loaded_count = 0;
                for entry in entries {
                    if entry.timestamp >= cutoff {
                        seen_map.insert(entry.id, entry.timestamp);
                        loaded_count += 1;
                    }
                }
                info!("[{}] Loaded {} recent IDs.", path_str, loaded_count);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("[{}] State file not found. Starting fresh.", path_str);
            }
            Err(e) => {
                warn!("[{}] Error reading state file: {}. Starting fresh.", path_str, e);
            }
        }
        Ok(())
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let seen_map = self.seen_ids.read().await;
        let entries: Vec<Entry> = seen_map
            .iter()
            .map(|(id, &ts)| Entry { 
                id: id.clone(),
                timestamp: ts, 
            })
            .collect();
        
        let data = serde_json::to_vec_pretty(&entries)?;
        tokio::fs::write(&self.file_path, data).await?;
        Ok(())
    }
}