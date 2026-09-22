use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub categories: Vec<String>,
}

pub fn scan_desktop_applications() -> Vec<AppInfo> {
    let mut apps = HashMap::new();

    for path in Iter::new(default_paths()) {
        if let Ok(bytes) = std::fs::read_to_string(&path) {
            if let Ok(entry) = DesktopEntry::from_str(&path, &bytes, None::<&[&str]>) {
                if entry.no_display() {
                    continue;
                }

                let id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();

                let name = entry.name(&[] as &[&str]).map(|s| s.to_string()).unwrap_or_else(|| id.clone());
                let exec = entry.exec().map(|s| s.to_string()).unwrap_or_default();
                let icon = entry.icon().map(|s| s.to_string());
                let comment = entry.comment(&[] as &[&str]).map(|s| s.to_string());
                let categories = entry
                    .categories()
                    .map(|c| c.into_iter().map(|s| s.to_string()).collect())
                    .unwrap_or_default();

                if !exec.is_empty() && !apps.contains_key(&id) {
                    apps.insert(
                        id.clone(),
                        AppInfo {
                            id,
                            name,
                            exec,
                            icon,
                            comment,
                            categories,
                        },
                    );
                }
            }
        }
    }

    let mut list: Vec<AppInfo> = apps.into_values().collect();
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    debug!("Discovered {} desktop applications.", list.len());
    list
}
