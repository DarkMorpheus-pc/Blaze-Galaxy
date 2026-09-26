use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolarConfig {
    #[serde(default)]
    pub shell: SolarShellConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
    #[serde(default)]
    pub taskbar: TaskbarConfig,
    #[serde(default)]
    pub shortcuts_hud: ShortcutsHudConfig,
    #[serde(default)]
    pub branding: BrandingConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

impl Default for SolarConfig {
    fn default() -> Self {
        Self {
            shell: SolarShellConfig::default(),
            performance: PerformanceConfig::default(),
            taskbar: TaskbarConfig::default(),
            shortcuts_hud: ShortcutsHudConfig::default(),
            branding: BrandingConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayConfig {
    /// Output scale factor for compositor (e.g. 0.75, 0.85, 1.0, 1.25, 1.5, 2.0)
    #[serde(default = "default_scale")]
    pub scale: f64,

    /// Text/Font scaling factor for GTK/Adwaita applications
    #[serde(default = "default_text_scale")]
    pub text_scale: f64,
}

fn default_scale() -> f64 {
    1.0
}

fn default_text_scale() -> f64 {
    1.0
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            scale: default_scale(),
            text_scale: default_text_scale(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskbarConfig {
    /// Whether the KDE-style bottom taskbar is enabled.
    /// Default is FALSE: Taskbar is optional, can be enabled in SolarUI settings!
    #[serde(default = "default_false")]
    pub enabled: bool,

    /// Theme mode: "dark" or "light"
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,

    /// Background opacity (0.5 to 1.0)
    #[serde(default = "default_opacity")]
    pub opacity: f64,

    /// Height in pixels (typically 40 - 56)
    #[serde(default = "default_height")]
    pub height: i32,

    /// Icon size in pixels (typically 18, 22, 28)
    #[serde(default = "default_icon_size")]
    pub icon_size: i32,

    /// Start menu icon: "solar-logo" (custom transparent bird) or "kickoff"
    #[serde(default = "default_start_menu_icon")]
    pub start_menu_icon: String,

    /// Show pinned launchers
    #[serde(default = "default_true")]
    pub show_pinned: bool,

    /// Show open window task buttons
    #[serde(default = "default_true")]
    pub show_tasks: bool,

    /// Show system tray buttons (Wifi, Volume, Power)
    #[serde(default = "default_true")]
    pub show_tray: bool,

    /// Show digital clock
    #[serde(default = "default_true")]
    pub show_clock: bool,

    /// Pinned application desktop IDs or commands
    #[serde(default = "default_pinned_apps")]
    pub pinned_apps: Vec<String>,
}

fn default_theme_mode() -> String {
    "dark".to_string()
}
fn default_opacity() -> f64 {
    0.92
}
fn default_height() -> i32 {
    44
}
fn default_icon_size() -> i32 {
    20
}
fn default_start_menu_icon() -> String {
    "solar-logo".to_string()
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_pinned_apps() -> Vec<String> {
    vec![
        "alacritty".to_string(),
        "nautilus".to_string(),
        "google-chrome-stable".to_string(),
    ]
}

impl Default for TaskbarConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Default is false: Taskbar is optional, user activates via settings
            theme_mode: default_theme_mode(),
            opacity: default_opacity(),
            height: default_height(),
            icon_size: default_icon_size(),
            start_menu_icon: default_start_menu_icon(),
            show_pinned: true,
            show_tasks: true,
            show_tray: true,
            show_clock: true,
            pinned_apps: default_pinned_apps(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShortcutsHudConfig {
    /// Whether to display the SolarUI welcome & shortcuts HUD at startup
    #[serde(default = "default_true")]
    pub show_at_startup: bool,
}

impl Default for ShortcutsHudConfig {
    fn default() -> Self {
        Self {
            show_at_startup: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrandingConfig {
    #[serde(default = "default_branding_name")]
    pub name: String,
    #[serde(default = "default_branding_title")]
    pub title: String,
    #[serde(default = "default_branding_subtitle")]
    pub subtitle: String,
    #[serde(default = "default_branding_signature")]
    pub signature: String,
}

fn default_branding_name() -> String {
    "SolarUI".to_string()
}
fn default_branding_title() -> String {
    "SolarUI Desktop Environment".to_string()
}
fn default_branding_subtitle() -> String {
    "KDE Plasma Serbestliği & Noctalia Bar Hibrit Tasarımı".to_string()
}
fn default_branding_signature() -> String {
    "Designed & Crafted for BlazeOS & CachyOS by DarkMorpheus".to_string()
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            name: default_branding_name(),
            title: default_branding_title(),
            subtitle: default_branding_subtitle(),
            signature: default_branding_signature(),
        }
    }
}

impl SolarConfig {
    pub fn config_path() -> PathBuf {
        crate::get_solar_config_dir().join("config.toml")
    }

    /// Missing configuration uses defaults; corrupt or unreadable configuration is an error.
    pub fn try_load() -> std::io::Result<Self> {
        Self::load_from(&Self::config_path())
    }

    fn load_from(path: &std::path::Path) -> std::io::Result<Self> {
        match fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }

    pub fn load() -> Self {
        Self::try_load().unwrap_or_else(|e| {
            tracing::error!("Cannot read SolarUI configuration: {e}");
            Self::default()
        })
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        let lock = crate::persistence::open_lock(&path.with_extension("lock"))?;
        lock.lock()?;
        // Do not silently overwrite an invalid file with fallback defaults.
        Self::load_from(&path)?;
        self.save_to(&path)
    }

    pub fn update(edit: impl FnOnce(&mut Self)) -> std::io::Result<()> {
        let path = Self::config_path();
        let lock = crate::persistence::open_lock(&path.with_extension("lock"))?;
        lock.lock()?;
        let mut config = Self::load_from(&path)?;
        edit(&mut config);
        config.save_to(&path)
    }

    fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        let content = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        crate::persistence::atomic_write(path, content.as_bytes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellEngine {
    Noctalia,
    Caelestia,
    Hybrid,
}

impl Default for ShellEngine {
    fn default() -> Self {
        ShellEngine::Noctalia
    }
}

impl ShellEngine {
    pub fn as_str(&self) -> &'static str {
        match self {
            ShellEngine::Noctalia => "noctalia",
            ShellEngine::Caelestia => "caelestia",
            ShellEngine::Hybrid => "hybrid",
        }
    }
}

impl std::fmt::Display for ShellEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ShellEngine {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "noctalia" | "noctalia-v5" | "native" => Ok(ShellEngine::Noctalia),
            "caelestia" | "quickshell" | "qml" => Ok(ShellEngine::Caelestia),
            "hybrid" | "mixed" | "surface-router" => Ok(ShellEngine::Hybrid),
            other => Err(format!(
                "Bilinmeyen kabuk motoru: '{}'. Geçerli seçenekler: noctalia, caelestia, hybrid",
                other
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SurfaceProvider {
    Noctalia,
    Caelestia,
    SolarUI,
}

impl Default for SurfaceProvider {
    fn default() -> Self {
        SurfaceProvider::Noctalia
    }
}

impl SurfaceProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            SurfaceProvider::Noctalia => "noctalia",
            SurfaceProvider::Caelestia => "caelestia",
            SurfaceProvider::SolarUI => "solarui",
        }
    }
}

impl std::fmt::Display for SurfaceProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SurfaceProvider {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "noctalia" | "noctalia-v5" => Ok(SurfaceProvider::Noctalia),
            "caelestia" | "quickshell" => Ok(SurfaceProvider::Caelestia),
            "solarui" | "native" => Ok(SurfaceProvider::SolarUI),
            other => Err(format!(
                "Bilinmeyen sağlayıcı: '{}'. Geçerli seçenekler: noctalia, caelestia, solarui",
                other
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceProfile {
    Legacy,
    Balanced,
    Ultra,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        PerformanceProfile::Balanced
    }
}

impl PerformanceProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            PerformanceProfile::Legacy => "legacy",
            PerformanceProfile::Balanced => "balanced",
            PerformanceProfile::Ultra => "ultra",
        }
    }
}

impl std::fmt::Display for PerformanceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for PerformanceProfile {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "legacy" | "low" | "battery" => Ok(PerformanceProfile::Legacy),
            "balanced" | "default" | "normal" => Ok(PerformanceProfile::Balanced),
            "ultra" | "high" | "full" => Ok(PerformanceProfile::Ultra),
            other => Err(format!(
                "Bilinmeyen performans profili: '{}'. Geçerli: legacy, balanced, ultra",
                other
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SolarShellConfig {
    #[serde(default)]
    pub engine: ShellEngine,
    #[serde(default = "default_surface_noctalia")]
    pub launcher_provider: SurfaceProvider,
    #[serde(default = "default_surface_caelestia")]
    pub dashboard_provider: SurfaceProvider,
    #[serde(default = "default_surface_noctalia")]
    pub bar_provider: SurfaceProvider,
    #[serde(default = "default_surface_noctalia")]
    pub notifications_provider: SurfaceProvider,
    #[serde(default = "default_surface_noctalia")]
    pub osd_provider: SurfaceProvider,
}

fn default_surface_noctalia() -> SurfaceProvider {
    SurfaceProvider::Noctalia
}
fn default_surface_caelestia() -> SurfaceProvider {
    SurfaceProvider::Caelestia
}

impl Default for SolarShellConfig {
    fn default() -> Self {
        Self {
            engine: ShellEngine::Noctalia,
            launcher_provider: SurfaceProvider::Noctalia,
            dashboard_provider: SurfaceProvider::Caelestia,
            bar_provider: SurfaceProvider::Noctalia,
            notifications_provider: SurfaceProvider::Noctalia,
            osd_provider: SurfaceProvider::Noctalia,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerformanceConfig {
    #[serde(default)]
    pub profile: PerformanceProfile,
    #[serde(default = "default_true")]
    pub blur: bool,
    #[serde(default = "default_true")]
    pub animations: bool,
    #[serde(default = "default_true")]
    pub shadows: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            profile: PerformanceProfile::Balanced,
            blur: true,
            animations: true,
            shadows: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrupt_configuration_is_distinguished_from_missing_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert!(SolarConfig::load_from(&path).is_ok());
        fs::write(&path, "[broken").unwrap();
        assert_eq!(
            SolarConfig::load_from(&path).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "[broken");
    }

    #[test]
    fn test_default_config() {
        let cfg = SolarConfig::default();
        assert!(
            !cfg.taskbar.enabled,
            "Taskbar must be disabled by default (only Noctalia bar)!"
        );
        assert_eq!(cfg.taskbar.theme_mode, "dark");
        assert_eq!(cfg.taskbar.start_menu_icon, "solar-logo");
        assert!(cfg.shortcuts_hud.show_at_startup);
        assert_eq!(cfg.branding.name, "SolarUI");
        assert_eq!(cfg.shell.engine, ShellEngine::Noctalia);
        assert_eq!(cfg.shell.launcher_provider, SurfaceProvider::Noctalia);
        assert_eq!(cfg.shell.dashboard_provider, SurfaceProvider::Caelestia);
        assert_eq!(cfg.performance.profile, PerformanceProfile::Balanced);

        let toml_str = toml::to_string(&cfg).expect("Serialize toml");
        let parsed: SolarConfig = toml::from_str(&toml_str).expect("Deserialize toml");
        assert_eq!(cfg, parsed);
    }

    #[test]
    fn test_engine_parsing_and_defaults() {
        assert_eq!(
            "noctalia".parse::<ShellEngine>().unwrap(),
            ShellEngine::Noctalia
        );
        assert_eq!(
            "caelestia".parse::<ShellEngine>().unwrap(),
            ShellEngine::Caelestia
        );
        assert_eq!(
            "hybrid".parse::<ShellEngine>().unwrap(),
            ShellEngine::Hybrid
        );
        assert!("invalid_engine".parse::<ShellEngine>().is_err());
    }

    #[test]
    fn test_surface_provider_parsing() {
        assert_eq!(
            "noctalia".parse::<SurfaceProvider>().unwrap(),
            SurfaceProvider::Noctalia
        );
        assert_eq!(
            "caelestia".parse::<SurfaceProvider>().unwrap(),
            SurfaceProvider::Caelestia
        );
        assert_eq!(
            "solarui".parse::<SurfaceProvider>().unwrap(),
            SurfaceProvider::SolarUI
        );
        assert!("invalid_provider".parse::<SurfaceProvider>().is_err());
    }

    #[test]
    fn test_performance_profile_parsing() {
        assert_eq!(
            "legacy".parse::<PerformanceProfile>().unwrap(),
            PerformanceProfile::Legacy
        );
        assert_eq!(
            "balanced".parse::<PerformanceProfile>().unwrap(),
            PerformanceProfile::Balanced
        );
        assert_eq!(
            "ultra".parse::<PerformanceProfile>().unwrap(),
            PerformanceProfile::Ultra
        );
        assert!("invalid_profile".parse::<PerformanceProfile>().is_err());
    }

    #[test]
    fn test_migration_and_partial_toml() {
        let partial_toml = r#"
            [shell]
            engine = "hybrid"
            launcher_provider = "caelestia"

            [performance]
            profile = "ultra"
        "#;

        let parsed: SolarConfig = toml::from_str(partial_toml).expect("Parse partial toml");
        assert_eq!(parsed.shell.engine, ShellEngine::Hybrid);
        assert_eq!(parsed.shell.launcher_provider, SurfaceProvider::Caelestia);
        // Omitted fields should receive their respective defaults!
        assert_eq!(parsed.shell.dashboard_provider, SurfaceProvider::Caelestia);
        assert_eq!(parsed.shell.bar_provider, SurfaceProvider::Noctalia);
        assert_eq!(parsed.performance.profile, PerformanceProfile::Ultra);
        assert!(parsed.performance.blur);
        assert!(!parsed.taskbar.enabled);
    }
}
