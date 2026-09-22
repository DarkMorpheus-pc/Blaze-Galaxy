pub use solar_common::config::{
    PerformanceConfig, PerformanceProfile, ShellEngine, SolarShellConfig, SurfaceProvider,
};

pub trait ShellEngineExt {
    fn display_title(&self) -> &'static str;
}

impl ShellEngineExt for ShellEngine {
    fn display_title(&self) -> &'static str {
        match self {
            ShellEngine::Noctalia => "Noctalia v5 (C++23 Native Wayland)",
            ShellEngine::Caelestia => "Caelestia (Quickshell QML)",
            ShellEngine::Hybrid => "SolarUI Hybrid (Noctalia Bar + Caelestia Surfaces)",
        }
    }
}
