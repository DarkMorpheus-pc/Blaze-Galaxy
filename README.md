# BlazeOS SolarEvolution 5 (Blaze-Galaxy)

<p align="center">
  <img src="https://raw.githubusercontent.com/DarkMorpheus-pc/Blaze-Galaxy/main/blazeos_custom_apps/usr/share/icons/hicolor/256x256/apps/solarui.png" alt="BlazeOS Logo" width="140" />
</p>

<h3 align="center">Next-Generation Hybrid Desktop Ecosystem & High-Performance Linux Distribution</h3>

<img width="1209" height="752" alt="image" src="https://github.com/user-attachments/assets/e888deac-5b91-4ea9-98e2-8026f75c3bcf" />
<img width="1208" height="757" alt="image" src="https://github.com/user-attachments/assets/f79b5aea-0fb7-4933-a5ca-b52321aa7ab0" />
<img width="1208" height="754" alt="image" src="https://github.com/user-attachments/assets/7b3e685d-8caf-4270-a9d6-da6324638f0b" />




<p align="center">
  <b>"From evolution to expansion — welcome to the galaxy."</b>
</p>

---

## About

**BlazeOS SolarEvolution 5** is a high-performance Linux operating system ecosystem built on Fedora Workstation, specifically optimized for gamers, developers, and content creators.

The system brings together the innovative **SolarUI (Niri Wayland + Noctalia + Caelestia)** desktop shell, a choice of 9 different desktop environments, and dual bootloader management (GRUB2 & Limine) in a unified setup.

---

## Key Features

- **SolarUI Desktop Ecosystem:** A clean, fluid desktop experience with glassmorphism support, built on the Niri Wayland window manager and the fast Noctalia shell and Caelestia shell.
- <img width="1205" height="753" alt="image" src="https://github.com/user-attachments/assets/beac43d9-0943-44c9-b951-712629f23456" />

- **9 Desktop Environment Options:**
  - **SolarUI:** The default fluid Niri + Noctalia desktop (Available Offline)
  - **GNOME:** Modern and stable GNOME 50 (Available Offline)
  - **Niri (Standalone):** A standalone window manager with infinite scrolling (Available Offline)
  - **KDE Plasma:** Highly customizable Plasma 6 on Wayland
  - **COSMIC:** An independent, modern, Rust-based desktop by System76
  - **Hyprland:** A dynamic tiling Wayland environment with fluid animations
  - **Cinnamon:** A classic and practical desktop layout
  - **XFCE:** An extremely lightweight and fast X11 environment
  - **Sway:** An i3-compatible, keyboard-driven tiling environment
- **Limine & GRUB2 Bootloader Support:** Switch between the Limine bootloader for lightning-fast startup and standard GRUB2 with Secure Boot support using a single command (`blaze-bootloader`).
- **BlazeOS Control Center (`blazeos-control`):** A system control center built using GTK4 / Libadwaita, featuring 5 tabs: Updates, Desktop, Performance, Tools, and About.
- **Low Latency & ZRAM Improvements:** DNF5 configurations for faster downloads, default ZRAM optimizations, and CachyOS kernel/animation tuning settings.
- **Online & Offline ISO Architecture:** Build support for a lightweight 3.5 GB Online ISO and a fully bundled 10 GB+ Offline ISO.

---

## Architecture

```text
Blaze-Galaxy / BlazeOS SolarEvolution 5
├── build_f45_final.sh          # 3.5 GB Online ISO build script
├── build_f45_offline.sh        # 10 GB+ Offline ISO build script
├── blazeos_custom_apps/        # Custom-developed system tools and desktop configurations
│   ├── usr/local/bin/
│   │   ├── blazeos-control     # GTK4 Control Center
│   │   ├── blazeos-welcome     # Welcome and initial setup wizard
│   │   ├── blaze-bootloader    # Limine / GRUB2 switching tool
│   │   ├── blazeos-postinstall # Post-installation automation service
│   │   └── blaze-optimize      # ZRAM & kernel optimizations
│   └── usr/bin/firehub         # FireHub smart wrapper
├── solarui/                    # SolarUI Rust components and source code
├── niri-src/                   # Niri Wayland compositor customizations
└── README.md
```

---

## 📦 Build Requirements

To build the ISO image locally, the following packages and tools must be installed on your system:

- **Operating System:** Fedora 40+, Arch Linux, CachyOS, or a RHEL-based 64-bit Linux distribution
- **Disk Space:** At least 30 GB of free tmpfs / disk space
- **Required Tools:**
  ```bash
  # On Fedora / RHEL:
  sudo dnf install -y xorriso isolinux squashfs-tools isomd5sum libattr-devel
  
  # On Arch Linux / CachyOS:
  sudo pacman -S --needed xorriso squashfs-tools attr
  ```

---

## How to Build

### 1. Clone the Repository

```bash
git clone https://github.com/DarkMorpheus-pc/Blaze-Galaxy.git
cd Blaze-Galaxy
```

### 2. Prepare the Base ISO Image

Place the Fedora Workstation 45 / 44 Live ISO image in the `f45_base/` directory.

### 3. Run the ISO Build Script

- **Online ISO Build (3.5 GB):**
  ```bash
  chmod +x build_f45_final.sh
  sudo ./build_f45_final.sh
  ```

- **Full Offline ISO Build (10 GB+):**
  ```bash
  chmod +x build_f45_offline.sh
  sudo ./build_f45_offline.sh
  ```

Once the build is complete, your ISO image will be available in the repository root directory (`Blaze-SolarEvolution-5-x86_64.iso`) and will pass the MD5 verification check (`checkisomd5 PASS`).

---

## ⌨️ SolarUI Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Mod + Return` | Open Terminal (Alacritty / Ptyxis) |
| `Mod + Space` | Open Application Launcher |
| `Mod + S` | Open SolarUI Control Center |
| `Mod + I` | Open Desktop Settings |
| `Mod + Alt + R` | Restart Desktop Shell (Supervisor) |
| `Mod + Alt + S` | Open QuickShell / SolarUI Quick Settings |
| `Mod + Q` / `Alt + F4` | Close Active Window |
| `Mod + Shift + E` | Log Out / Open Exit Menu |

---

## 📄 License & Contributing

This project is released under the **GPL-3.0** license. You can contribute by opening a pull request or reporting an issue.

Designed & Crafted for **BlazeOS & CachyOS** by **DarkMorpheus**.
