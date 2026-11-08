//! SigmaOS-theme-sync — Cross-application theme synchronization for SigmaOS.

use std::collections::HashMap;

/// Theme sync target — an app that receives theme config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncTarget {
    pub app_class: String,
    pub config_file: String,
    pub enabled: bool,
    pub last_synced: Option<u64>,
}

/// Theme colors palette.
#[derive(Debug, Clone, Default)]
pub struct ThemePalette {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub error: String,
    pub warning: String,
    pub success: String,
}

/// Sync state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    Idle,
    Syncing,
    Synced,
    Failed,
}

/// Sync report for one target.
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub app_class: String,
    pub success: bool,
    pub config_generated: bool,
    pub error: Option<String>,
}

/// Theme sync daemon.
#[derive(Debug)]
pub struct ThemeSyncDaemon {
    targets: Vec<SyncTarget>,
    palette: ThemePalette,
    state: SyncState,
    synced_count: usize,
    failed_count: usize,
}

impl Default for ThemeSyncDaemon {
    fn default() -> Self { Self::new() }
}

impl ThemeSyncDaemon {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            palette: ThemePalette::default(),
            state: SyncState::Idle,
            synced_count: 0,
            failed_count: 0,
        }
    }

    /// Register a sync target.
    pub fn register_target(&mut self, app: &str, path: &str) {
        self.targets.push(SyncTarget {
            app_class: app.into(),
            config_file: path.into(),
            enabled: true,
            last_synced: None,
        });
    }

    /// Register target with enabled flag.
    pub fn register_target_enabled(&mut self, app: &str, path: &str, enabled: bool) {
        self.targets.push(SyncTarget {
            app_class: app.into(),
            config_file: path.into(),
            enabled,
            last_synced: None,
        });
    }

    /// Remove target by app class.
    pub fn remove_target(&mut self, app: &str) -> bool {
        let before = self.targets.len();
        self.targets.retain(|t| t.app_class != app);
        self.targets.len() < before
    }

    /// Set theme palette.
    pub fn set_palette(&mut self, palette: ThemePalette) {
        self.palette = palette;
    }

    /// Get current palette.
    pub fn palette(&self) -> &ThemePalette { &self.palette }

    /// Enable/disable a target.
    pub fn set_target_enabled(&mut self, app: &str, enabled: bool) -> bool {
        if let Some(t) = self.targets.iter_mut().find(|t| t.app_class == app) {
            t.enabled = enabled;
            return true;
        }
        false
    }

    /// Generate config content for a target app.
    pub fn generate_config(&self, app: &str) -> Option<String> {
        let target = self.targets.iter().find(|t| t.app_class == app)?;
        if !target.enabled { return None; }
        Some(format!(
            "# SigmaOS theme sync for {}\n[colors]\nprimary = \"{}\"\nbackground = \"{}\"\nforeground = \"{}\"\naccent = \"{}\"\n",
            app, self.palette.primary, self.palette.background, self.palette.foreground, self.palette.accent
        ))
    }

    /// Sync all enabled targets.
    pub fn sync_all(&mut self) -> Vec<SyncReport> {
        self.state = SyncState::Syncing;
        let mut reports = Vec::new();
        let now = now_secs();

        for i in 0..self.targets.len() {
            let target = &mut self.targets[i];
            if !target.enabled {
                reports.push(SyncReport {
                    app_class: target.app_class.clone(),
                    success: true,
                    config_generated: false,
                    error: None,
                });
                continue;
            }
            let ok = !target.config_file.is_empty();
            target.last_synced = Some(now);
            if ok {
                self.synced_count += 1;
            } else {
                self.failed_count += 1;
            }
            reports.push(SyncReport {
                app_class: target.app_class.clone(),
                success: ok,
                config_generated: ok,
                error: if ok { None } else { Some("empty config path".into()) },
            });
        }
        self.state = if self.failed_count > 0 && self.synced_count == 0 {
            SyncState::Failed
        } else {
            SyncState::Synced
        };
        reports
    }

    /// Get state.
    pub fn state(&self) -> SyncState { self.state }
    /// Target count.
    pub fn target_count(&self) -> usize { self.targets.len() }
    /// Enabled target count.
    pub fn enabled_count(&self) -> usize { self.targets.iter().filter(|t| t.enabled).count() }
    /// Synced count.
    pub fn synced_count(&self) -> usize { self.synced_count }
    /// Failed count.
    pub fn failed_count(&self) -> usize { self.failed_count }

    /// List targets.
    pub fn list_targets(&self) -> &[SyncTarget] { &self.targets }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette() -> ThemePalette {
        ThemePalette {
            primary: "#ff0000".into(),
            secondary: "#00ff00".into(),
            background: "#000000".into(),
            foreground: "#ffffff".into(),
            accent: "#0000ff".into(),
            error: "#ff0000".into(),
            warning: "#ffff00".into(),
            success: "#00ff00".into(),
        }
    }

    #[test]
    fn test_register_targets() {
        let mut sync = ThemeSyncDaemon::new();
        sync.register_target("Alacritty", "~/.config/alacritty/alacritty.toml");
        sync.register_target("Foot", "~/.config/foot/foot.ini");
        assert_eq!(sync.target_count(), 2);
        assert_eq!(sync.enabled_count(), 2);
    }

    #[test]
    fn test_remove_target() {
        let mut sync = ThemeSyncDaemon::new();
        sync.register_target("Alacritty", "/cfg/a.toml");
        sync.register_target("Foot", "/cfg/f.ini");
        assert!(sync.remove_target("Alacritty"));
        assert_eq!(sync.target_count(), 1);
        assert!(!sync.remove_target("Nonexistent"));
    }

    #[test]
    fn test_disable_target() {
        let mut sync = ThemeSyncDaemon::new();
        sync.register_target("Alacritty", "/cfg/a.toml");
        assert!(sync.set_target_enabled("Alacritty", false));
        assert_eq!(sync.enabled_count(), 0);
    }

    #[test]
    fn test_generate_config() {
        let mut sync = ThemeSyncDaemon::new();
        sync.set_palette(palette());
        sync.register_target("Alacritty", "/cfg/a.toml");
        
        let cfg = sync.generate_config("Alacritty").unwrap();
        assert!(cfg.contains("primary"));
        assert!(cfg.contains("#ff0000"));
        assert!(cfg.contains("Alacritty"));
    }

    #[test]
    fn test_generate_config_disabled() {
        let mut sync = ThemeSyncDaemon::new();
        sync.register_target("Alacritty", "/cfg/a.toml");
        sync.set_target_enabled("Alacritty", false);
        assert!(sync.generate_config("Alacritty").is_none());
    }

    #[test]
    fn test_sync_all() {
        let mut sync = ThemeSyncDaemon::new();
        sync.register_target("Alacritty", "/cfg/a.toml");
        sync.register_target("Foot", "/cfg/f.ini");
        
        let reports = sync.sync_all();
        assert_eq!(reports.len(), 2);
        assert!(reports.iter().all(|r| r.success));
        assert_eq!(sync.state(), SyncState::Synced);
        assert_eq!(sync.synced_count(), 2);
    }

    #[test]
    fn test_sync_failure() {
        let mut sync = ThemeSyncDaemon::new();
        sync.register_target("Broken", ""); // empty path = failure
        
        let reports = sync.sync_all();
        assert_eq!(reports.len(), 1);
        assert!(!reports[0].success);
        assert_eq!(sync.failed_count(), 1);
        assert_eq!(sync.state(), SyncState::Failed);
    }

    #[test]
    fn test_last_synced_timestamp() {
        let mut sync = ThemeSyncDaemon::new();
        sync.register_target("Alacritty", "/cfg/a.toml");
        sync.sync_all();
        
        let targets = sync.list_targets();
        assert!(targets[0].last_synced.is_some());
    }
}
