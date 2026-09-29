# SolarUI

Next-Generation Hybrid Desktop Environment for Linux.

SolarUI is a high-performance, modular desktop ecosystem designed to combine the fluidity and infinite-scrolling ergonomics of the Niri Wayland compositor with a modern, dual-engine shell architecture (Noctalia and Caelestia) managed by a unified Rust supervisor.

---

## Architecture Overview

SolarUI operates on a multi-layer decoupled architecture:

1. Compositor Layer: Niri scrollable-tiling Wayland compositor providing hardware-accelerated animations, spring physics, dynamic column tiling, and layer-shell integration.
2. Core Supervisor Layer (Rust):
   - solar-core: Manages window registration, session tracking, IPC dispatching, and state caching.
   - solar-shell: Acts as the desktop shell runtime supervisor. Controls live switching between Noctalia and Caelestia shell backends with automatic health check and crash rollback protection.
   - solar-session: Wayland session bootstrap executable. Prepares runtime environment variables, enforces desktop branding, and launches the compositor with SolarUI configurations.
   - solar-niri: Asynchronous client implementation of Niri IPC protocol.
   - solar-common: Shared data structures, configuration parsing, IPC event definitions, and hardware performance profiling.
   - solar-adapter: Distribution and runtime detection utilities.
3. Shell Engine Backends:
   - Noctalia: Native C++ Wayland desktop shell providing unified top status bars, taskbars, application launchers, volume/brightness OSDs, and system tray integration.
   - Caelestia: High-fidelity QML/Qt6 desktop shell offering collapsible sidebars, media drawers, system status monitors, and extensive localization support.
4. Branding and Interception:
   - libsolar_brand.so: Lightweight LD_PRELOAD interceptor that harmonizes toolkit strings and system labels under the SolarUI desktop identity.

---

## Target Operating Systems

SolarUI is built for Linux platforms supporting modern Wayland display servers:

- Fedora Linux: Fedora 40, 41, 42+ (Primary distribution platform for BlazeOS)
- Arch Linux / CachyOS: Full rolling-release support
- openSUSE: Tumbleweed / Slowroll
- Ubuntu / Debian: Debian 13 (Trixie)+, Ubuntu 24.04 LTS+ (Wayland session required)

Display Server Requirement: Native Wayland environment. X11 standalone compositor mode is not supported. Legacy X11 applications run seamlessly through XWayland / xwayland-satellite.

---

## System Requirements

### Minimum Hardware Requirements
- Processor: 64-bit dual-core x86_64 or ARM64 processor (1.8 GHz or faster)
- Memory: 2 GB RAM (SolarUI idle footprint is approximately 400 MB to 650 MB depending on shell engine)
- Graphics: GPU with OpenGL ES 3.0 or Vulkan 1.1 support
  - Intel HD Graphics 4000 series or newer
  - AMD GCN 1.0 (HD 7000) or newer
  - NVIDIA Kepler (GTX 600) or newer (open or proprietary driver with Wayland support)
- Storage: 1.5 GB free disk space for binaries, QML plugins, and default assets
- Display: 1024x768 display resolution

### Recommended Hardware Requirements
- Processor: Quad-core modern processor (Intel Core i3/i5/i7/i9 8th Gen+, AMD Ryzen 2000+)
- Memory: 4 GB RAM or more
- Graphics: Dedicated GPU with full Vulkan and Wayland DMA-BUF support
  - Intel UHD 620 / Iris Xe / Arc
  - AMD Radeon RX 400 series or newer
  - NVIDIA GeForce GTX 10 series or newer with driver version 555+
- Storage: 5 GB SSD storage
- Display: 1920x1080 (Full HD) or higher resolution with HiDPI / fractional scaling support

---

## Dependencies

### Build Dependencies
- Rust Toolchain: Rust 1.80+ (cargo, rustc)
- C/C++ Compilers: GCC 13+ or Clang 16+
- Build Systems: CMake 3.25+, Meson 1.2+, Ninja
- Libraries and Development Headers:
  - Wayland: wayland-client, wayland-server, wayland-protocols
  - Compositor and Display: libseat, libxkbcommon, libinput
  - GUI and Rendering: Pango, Cairo, EGL, OpenGL ES, Qt6 (Base, Declarative/QML)
  - Audio and Session: PipeWire, WirePlumber, systemd / elogind

### Distribution Package Installation Commands

#### Fedora / RHEL
```bash
sudo dnf install -y \
    cargo rust gcc gcc-c++ cmake meson ninja-build \
    wayland-devel wayland-protocols-devel libseat-devel \
    libxkbcommon-devel pango-devel cairo-devel \
    qt6-qtbase-devel qt6-qtdeclarative-devel \
    pipewire-devel pulseaudio-libs-devel
```

#### Arch Linux / CachyOS
```bash
sudo pacman -S --needed \
    rust gcc cmake meson ninja \
    wayland wayland-protocols libseat \
    libxkbcommon pango cairo \
    qt6-base qt6-declarative \
    pipewire libpulse
```

---

## Compilation and Installation

### 1. Clone the Repository
```bash
git clone https://github.com/DarkMorpheus-pc/SolarUI.git
cd SolarUI
```

### 2. Build Rust Workspace
Compile the core supervisor and session binaries:
```bash
cargo build --workspace --release
```

Run internal automated verification tests (40 tests):
```bash
cargo test --workspace --release
```

### 3. Build Branding Hook (Optional)
```bash
gcc -shared -fPIC -O2 -o libsolar_brand.so libsolar_brand.c -ldl
```

### 4. Install Binaries and System Data
Install the compiled binaries, configurations, and Wayland session files:

```bash
# Core binaries
sudo install -Dm755 target/release/solar-core /usr/bin/solar-core
sudo install -Dm755 target/release/solar-shell /usr/bin/solar-shell
sudo install -Dm755 target/release/solar-session /usr/bin/solar-session

# Session entry
sudo install -Dm644 data/solarui.desktop /usr/share/wayland-sessions/solarui.desktop

# Default configuration template
sudo mkdir -p /etc/solarui
sudo install -Dm644 data/niri.kdl /etc/solarui/niri.kdl

# Branding hook
sudo mkdir -p /usr/lib/solarui
sudo install -Dm755 libsolar_brand.so /usr/lib/solarui/libsolar_brand.so
```

---

## Usage and Operations

### Starting the Session
1. Display Manager: Select "SolarUI" from your display manager session list (SDDM or GDM).
2. TTY / Direct Execution:
   ```bash
   solar-session
   ```

### Managing Shell Engines
SolarUI features live switching between shell providers without restarting the compositor:

Switch to Caelestia Shell:
```bash
solar-shell switch caelestia
```

Switch to Noctalia Shell:
```bash
solar-shell switch noctalia
```

Inspect Current Supervisor and Engine Status:
```bash
solar-shell status
```

### Shell Action Routing
SolarUI CLI provides unified actions mapped across whichever shell engine is currently running:

- Open Application Launcher:
  ```bash
  solar-shell route launcher
  ```
- Open Control Center:
  ```bash
  solar-shell route control-center
  ```
- Open Settings Panel:
  ```bash
  solar-shell route settings
  ```
- Open System Dashboard:
  ```bash
  solar-shell route dashboard
  ```
- Trigger Screen Lock:
  ```bash
  solar-shell route session
  ```

---

## Default Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| Mod + Space | Open Solar Omnibar / Application Search |
| Mod + G | Open Blaze GameZone (Gaming & Store Shell) |
| Mod + Return | Open Primary Terminal Emulator |
| Mod + Q | Close Focused Window |
| Mod + F | Maximize Focused Column |
| Mod + Shift + F | Toggle Fullscreen Window |
| Mod + S | Open Control Center |
| Mod + D | Toggle Dashboard |
| Mod + I | Open Desktop Settings |
| Mod + L | Lock Screen |
| Mod + Left / Right | Focus Adjacent Column |
| Mod + Up / Down | Focus Window Up / Down |
| Mod + Shift + Left / Right | Move Column Left / Right |
| Mod + 1 ... 5 | Switch to Workspace 1 to 5 |
| Mod + Shift + 1 ... 5 | Move Focused Window to Workspace 1 to 5 |
| Print | Interactive Screenshot Selection |

---

## Configuration Files

- Primary Compositor Configuration:
  - System default: `/etc/solarui/niri.kdl`
  - User override: `~/.config/solarui/niri.kdl` or `~/.config/niri/config.kdl`
- Shell Engine State & Supervisor Config:
  - Configuration: `~/.config/solarui/config.toml`
  - Engine Selection: `desired_engine = "caelestia"` or `desired_engine = "noctalia"`
- Noctalia Settings:
  - `~/.local/state/noctalia/settings.toml`
- Caelestia Shell Settings:
  - `~/.config/quickshell/caelestia/`

---

## License

SolarUI is free software licensed under the GNU General Public License v3.0 or later (GPL-3.0-or-later). See the LICENSE file for details.
