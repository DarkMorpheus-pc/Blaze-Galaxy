use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinimizedWindowRecord {
    pub orig_workspace_id: u64,
    pub was_floating: bool,
    #[serde(default)]
    pub parked_workspace_id: Option<u64>,
    // Old versions moved floating windows down by 1000 pixels before parking.
    #[serde(default = "legacy_translation")]
    pub translated: bool,
}
fn legacy_translation() -> bool {
    true
}

pub struct MinimizedStore;

/// Holds the cross-process lock for the entire compositor operation.
/// Never block the UI waiting for another minimize/restore transaction.
pub struct MinimizedTransaction {
    _lock: File,
    path: PathBuf,
    records: HashMap<u64, MinimizedWindowRecord>,
}

impl MinimizedStore {
    pub fn file_path() -> PathBuf {
        crate::paths::get_solar_runtime_dir().join("minimized_windows.json")
    }

    pub fn load() -> HashMap<u64, MinimizedWindowRecord> {
        read_records(&Self::file_path()).unwrap_or_else(|e| {
            tracing::warn!("Cannot read minimized window records: {e}");
            HashMap::new()
        })
    }

    pub fn transaction() -> io::Result<MinimizedTransaction> {
        Self::transaction_at(&Self::file_path())
    }

    pub fn transaction_at(path: &Path) -> io::Result<MinimizedTransaction> {
        let lock = crate::persistence::try_lock(&path.with_extension("lock"))?;
        let records = read_records(path)?;
        Ok(MinimizedTransaction {
            _lock: lock,
            path: path.to_owned(),
            records,
        })
    }
}

impl MinimizedTransaction {
    pub fn get(&self, id: u64) -> Option<MinimizedWindowRecord> {
        self.records.get(&id).copied()
    }
    pub fn insert(&mut self, id: u64, record: MinimizedWindowRecord) -> io::Result<()> {
        self.records.insert(id, record);
        self.save()
    }
    pub fn remove(&mut self, id: u64) -> io::Result<()> {
        self.records.remove(&id);
        self.save()
    }
    fn save(&self) -> io::Result<()> {
        let data = serde_json::to_vec(&self.records).map_err(io::Error::other)?;
        crate::persistence::atomic_write(&self.path, &data)
    }
}

fn read_records(path: &Path) -> io::Result<HashMap<u64, MinimizedWindowRecord>> {
    let content = match std::fs::read(path) {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(e) => return Err(e),
    };
    if let Ok(records) = serde_json::from_slice(&content) {
        return Ok(records);
    }
    // Migrate the original id -> workspace format.
    let old: HashMap<u64, u64> = serde_json::from_slice(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(old
        .into_iter()
        .map(|(id, ws)| {
            (
                id,
                MinimizedWindowRecord {
                    orig_workspace_id: ws,
                    was_floating: false,
                    parked_workspace_id: None,
                    translated: true,
                },
            )
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concurrent_transactions_cannot_lose_records() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("windows.json");
        let mut first = MinimizedStore::transaction_at(&path).unwrap();
        assert!(MinimizedStore::transaction_at(&path).is_err());
        first
            .insert(
                1,
                MinimizedWindowRecord {
                    orig_workspace_id: 900,
                    was_floating: true,
                    parked_workspace_id: Some(901),
                    translated: false,
                },
            )
            .unwrap();
        drop(first);
        let mut second = MinimizedStore::transaction_at(&path).unwrap();
        assert_eq!(second.get(1).unwrap().orig_workspace_id, 900);
        second.insert(2, second.get(1).unwrap()).unwrap();
        assert_eq!(read_records(&path).unwrap().len(), 2);
    }
    #[test]
    fn corrupt_state_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("windows.json");
        std::fs::write(&path, b"broken").unwrap();
        assert!(MinimizedStore::transaction_at(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"broken");
    }
    #[test]
    fn old_records_remain_restorable() {
        let record: MinimizedWindowRecord =
            serde_json::from_str(r#"{"orig_workspace_id":42,"was_floating":true}"#).unwrap();
        assert!(record.translated);
    }
}
