# dms-awww

Efficient wallpaper management for [Dank Material Shell (DMS)](https://github.com/AvengeMedia/DankMaterialShell) using [awww](https://codeberg.org/LGFae/awww).

## Important: Before Installing

**You MUST disable DMS's built-in wallpaper system first:**

1. Open DMS Settings → Wallpaper
2. Find "External Wallpaper Management" section
3. Enable **"Disable Built-in Wallpapers"** toggle

This tells DMS to let external wallpaper tools (like awww) handle wallpapers instead of its VRAM-based system.

## How It Works

- Monitors DMS's `session.json` for wallpaper changes
- Applies wallpapers via `awww` (no VRAM usage)
- Triggers DMS's matugen for theme generation

## Prerequisites

- DMS (with "Disable Built-in Wallpapers" enabled)
- awww daemon running (`awww-daemon &`)
- Rust toolchain (required to build the binary)

### Installing Rust

**Arch Linux / Manjaro:**
```bash
sudo pacman -S rust
```

**Other distributions (via rustup):**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

**Verify installation:**
```bash
cargo --version
```

## Installation

```bash
git clone https://github.com/higorprado/dms-awww-integration.git
cd dms-awww-integration
./install.sh
```

Enable the service:
```bash
systemctl --user enable dms-awww.service
systemctl --user start dms-awww.service
```

## Usage

Change wallpapers through DMS as normal. The daemon detects changes and applies them via awww automatically.

## Troubleshooting

**Service not working:**
```bash
systemctl --user status dms-awww.service
journalctl --user -u dms-awww.service -n 50
```

**awww errors:** Make sure `awww-daemon` is running before starting the service.

## Uninstallation

```bash
./uninstall.sh
```
