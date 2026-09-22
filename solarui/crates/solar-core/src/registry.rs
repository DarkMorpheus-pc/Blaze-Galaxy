use solar_common::{MinimizedStore, SolarWindowInfo, SolarWindowState, WindowMode};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone, Default)]
pub struct SolarWindowRegistry {
    windows: HashMap<u64, SolarWindowState>,
    active_workspace: u64,
}

impl SolarWindowRegistry {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            active_workspace: 1,
        }
    }

    /// Synchronizes the registry with canonical window list from Niri.
    /// Preserves minimization state and transitions window lifecycle state.
    pub fn update_from_niri(
        &mut self,
        niri_windows: Vec<SolarWindowInfo>,
        active_workspace: u64,
    ) -> Vec<SolarWindowState> {
        self.update_with_records(niri_windows, active_workspace, MinimizedStore::load())
    }

    fn update_with_records(
        &mut self,
        niri_windows: Vec<SolarWindowInfo>,
        active_workspace: u64,
        mut minimized_map: HashMap<u64, solar_common::MinimizedWindowRecord>,
    ) -> Vec<SolarWindowState> {
        self.active_workspace = active_workspace;

        // 1. Collect all active window IDs from Niri
        let active_ids: Vec<u64> = niri_windows.iter().map(|w| w.id).collect();
        // A snapshot may be stale while a minimize operation is in progress.
        // Only filter this view; never delete persistent records from a snapshot.
        minimized_map.retain(|id, _| active_ids.contains(id));

        // 2. Remove closed windows from registry
        self.windows.retain(|id, _| active_ids.contains(id));

        // 3. Upsert / reconcile states
        for niri_win in niri_windows {
            let id = niri_win.id;
            // Restore events can arrive before the transaction removes its record.
            // The compositor location, not record existence alone, determines visibility.
            let is_minimized = minimized_map.get(&id).is_some_and(|record| {
                record.parked_workspace_id.map_or(
                    niri_win.workspace_id != Some(record.orig_workspace_id) && !niri_win.is_focused,
                    |parked| niri_win.workspace_id == Some(parked),
                )
            });
            let orig_workspace = minimized_map
                .get(&id)
                .map(|r| r.orig_workspace_id)
                .unwrap_or_else(|| niri_win.workspace_id.unwrap_or(active_workspace));

            let mode = if is_minimized {
                WindowMode::Minimized { orig_workspace }
            } else if niri_win.is_floating {
                WindowMode::Floating
            } else {
                WindowMode::Normal
            };

            let state = SolarWindowState {
                id,
                title: niri_win.title,
                app_id: niri_win.app_id,
                workspace_id: if is_minimized {
                    orig_workspace
                } else {
                    niri_win.workspace_id.unwrap_or(active_workspace)
                },
                output_id: None,
                mode,
                is_focused: niri_win.is_focused,
                is_urgent: false,
            };

            self.windows.insert(id, state);
        }

        debug!(
            "SolarWindowRegistry updated: {} windows in registry (active ws: {})",
            self.windows.len(),
            self.active_workspace
        );

        self.to_vec()
    }

    /// Sets the active workspace id
    pub fn set_active_workspace(&mut self, ws_id: u64) {
        self.active_workspace = ws_id;
    }

    pub fn get_window(&self, id: u64) -> Option<&SolarWindowState> {
        self.windows.get(&id)
    }

    pub fn get_focused_window(&self) -> Option<&SolarWindowState> {
        self.windows.values().find(|w| w.is_focused)
    }

    pub fn to_vec(&self) -> Vec<SolarWindowState> {
        let mut list: Vec<SolarWindowState> = self.windows.values().cloned().collect();
        list.sort_by_key(|w| w.id);
        list
    }

    /// Returns taskbar-visible windows for a given workspace.
    /// Excludes system components (shell, panel) and includes minimized windows from that workspace.
    pub fn get_tasks_for_workspace(&self, ws_id: u64) -> Vec<SolarWindowState> {
        let mut tasks: Vec<SolarWindowState> = self
            .windows
            .values()
            .filter(|win| {
                // Ignore system panels
                if let Some(ref app_id) = win.app_id {
                    let id_lower = app_id.to_lowercase();
                    if id_lower.contains("solar-shell")
                        || id_lower.contains("solarui")
                        || id_lower.contains("noctalia")
                    {
                        return false;
                    }
                }
                if let Some(ref title) = win.title {
                    let title_lower = title.to_lowercase();
                    if title_lower.contains("solarui taskbar") || title_lower.contains("solarshell")
                    {
                        return false;
                    }
                }

                // Check workspace or original workspace if minimized
                match win.mode {
                    WindowMode::Minimized { orig_workspace } => orig_workspace == ws_id,
                    _ => win.workspace_id == ws_id,
                }
            })
            .cloned()
            .collect();

        tasks.sort_by_key(|w| w.id);
        tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restored_window_is_visible_before_recovery_record_is_removed() {
        let mut reg = SolarWindowRegistry::new();
        let record = solar_common::MinimizedWindowRecord {
            orig_workspace_id: 900,
            parked_workspace_id: Some(901),
            was_floating: false,
            translated: false,
        };
        let records = HashMap::from([(42, record)]);
        let window = SolarWindowInfo {
            id: 42,
            workspace_id: Some(901),
            ..Default::default()
        };
        let parked = reg.update_with_records(vec![window.clone()], 900, records.clone());
        assert!(matches!(parked[0].mode, WindowMode::Minimized { .. }));
        let restored = reg.update_with_records(
            vec![SolarWindowInfo {
                workspace_id: Some(777),
                ..window
            }],
            777,
            records,
        );
        assert_eq!(restored[0].mode, WindowMode::Normal);
        assert_eq!(restored[0].workspace_id, 777);
    }

    #[test]
    fn test_window_registry_sync() {
        let mut reg = SolarWindowRegistry::new();
        let windows = vec![
            SolarWindowInfo {
                id: 101,
                title: Some("Alacritty".into()),
                app_id: Some("alacritty".into()),
                is_focused: true,
                is_floating: false,
                workspace_id: Some(1),
            },
            SolarWindowInfo {
                id: 102,
                title: Some("Google Chrome".into()),
                app_id: Some("google-chrome".into()),
                is_focused: false,
                is_floating: false,
                workspace_id: Some(1),
            },
        ];

        let state = reg.update_with_records(windows, 1, HashMap::new());
        assert_eq!(state.len(), 2);
        assert_eq!(reg.get_tasks_for_workspace(1).len(), 2);
        assert_eq!(reg.get_focused_window().map(|w| w.id), Some(101));

        // When window 101 closes in Niri
        let windows_after = vec![SolarWindowInfo {
            id: 102,
            title: Some("Google Chrome".into()),
            app_id: Some("google-chrome".into()),
            is_focused: true,
            is_floating: false,
            workspace_id: Some(1),
        }];
        let state_after = reg.update_with_records(windows_after, 1, HashMap::new());
        assert_eq!(state_after.len(), 1);
        assert_eq!(reg.get_window(101), None);
        assert_eq!(reg.get_focused_window().map(|w| w.id), Some(102));
    }
}
