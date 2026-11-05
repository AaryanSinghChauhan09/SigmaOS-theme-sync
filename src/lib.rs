#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncTarget {
    pub app_class: String,
    pub config_file: String,
}

#[derive(Debug, Default)]
pub struct ThemeSyncDaemon {
    targets: Vec<SyncTarget>,
}

impl ThemeSyncDaemon {
    pub fn new() -> Self { Self { targets: Vec::new() } }
    pub fn register_target(&mut self, app: &str, path: &str) {
        self.targets.push(SyncTarget { app_class: app.into(), config_file: path.into() });
    }
    pub fn target_count(&self) -> usize { self.targets.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_theme_sync_daemon_aaa() {
        // Arrange
        let mut sync = ThemeSyncDaemon::new();
        // Act
        sync.register_target("Alacritty", "~/.config/alacritty/alacritty.toml");
        sync.register_target("Foot", "~/.config/foot/foot.ini");
        // Assert
        assert_eq!(sync.target_count(), 2);
    }
}
