use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShellAction {
    Launcher,
    ControlCenter,
    Dashboard,
    Session,
    Settings,
    Overview,
    WindowSwitcher,
}

impl ShellAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ShellAction::Launcher => "launcher",
            ShellAction::ControlCenter => "control-center",
            ShellAction::Dashboard => "dashboard",
            ShellAction::Session => "session",
            ShellAction::Settings => "settings",
            ShellAction::Overview => "overview",
            ShellAction::WindowSwitcher => "window-switcher",
        }
    }
}

impl fmt::Display for ShellAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ShellAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "launcher" | "apps" | "app-launcher" | "rofi" | "fuzzel" => Ok(ShellAction::Launcher),
            "control-center" | "cc" | "control" | "quick-settings" => Ok(ShellAction::ControlCenter),
            "dashboard" | "dash" | "widgets" => Ok(ShellAction::Dashboard),
            "session" | "power" | "logout" | "exit" => Ok(ShellAction::Session),
            "settings" | "config" | "preferences" => Ok(ShellAction::Settings),
            "overview" => Ok(ShellAction::Overview),
            "window-switcher" | "switcher" | "alt-tab" | "tasks" => Ok(ShellAction::WindowSwitcher),
            other => Err(format!(
                "Geçersiz aksiyon: '{}'. Desteklenenler: launcher, control-center, dashboard, session, settings, overview, window-switcher",
                other
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_parsing() {
        assert_eq!("launcher".parse::<ShellAction>().unwrap(), ShellAction::Launcher);
        assert_eq!("control-center".parse::<ShellAction>().unwrap(), ShellAction::ControlCenter);
        assert_eq!("dashboard".parse::<ShellAction>().unwrap(), ShellAction::Dashboard);
        assert_eq!("session".parse::<ShellAction>().unwrap(), ShellAction::Session);
        assert_eq!("settings".parse::<ShellAction>().unwrap(), ShellAction::Settings);
        assert_eq!("overview".parse::<ShellAction>().unwrap(), ShellAction::Overview);
        assert_eq!("alt-tab".parse::<ShellAction>().unwrap(), ShellAction::WindowSwitcher);
        assert!("dangerous; rm -rf /".parse::<ShellAction>().is_err());
    }
}
