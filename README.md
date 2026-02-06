# dms-awww-integration

Efficient wallpaper management for [Dank Material Shell (DMS)](https://github.com/AvengeMedia/DankMaterialShell) using [awww](https://codeberg.org/LGFae/awww).

## Problem

DMS stores wallpapers in VRAM for quick transitions and effects. While this provides smooth animations, it consumes significant GPU memory (VRAM). The [awww](https://codeberg.org/LGFae/awww) tool provides a more memory-efficient alternative for setting wallpapers by writing directly to the Wayland compositor without storing images in VRAM.

## Solution

This project provides a Rust daemon that:
1. Monitors DMS's `session.json` for wallpaper changes using inotify (event-driven)
2. Applies the new wallpaper via `awww` (efficient, no VRAM usage)
3. Triggers DMS's matugen to regenerate theme colors in parallel
4. Auto-detects Niri monitors

This gives you the best of both worlds: DMS's UI and theming with awww's efficient wallpaper rendering.

## Features

- **Event-driven monitoring** - Uses inotify for instant detection (<10ms vs 1s polling)
- **Parallel execution** - awww and matugen run concurrently
- **Zero idle CPU** - Event-driven architecture, no polling
- **Auto-detection** - Automatically detects Niri monitor outputs
- **Proper JSON parsing** - Uses serde for reliable configuration handling
- **Configuration file** - TOML/YAML configuration support
- **Environment variable overrides** - `DMS_AWWW_*` prefix
- **Comprehensive logging** - Structured logging with tracing
- **VRAM savings** - ~100-500MB compared to DMS wallpaper caching

## Prerequisites

- **Dank Material Shell (DMS)** - Installed and configured ([Github](https://github.com/AvengeMedia/DankMaterialShell))
- **awww** - Wallpaper utility for Niri/Hyprland ([Codeberg](https://codeberg.org/LGFae/awww))
- **matugen** - Material Design color generator (typically comes with DMS)
- **Niri** - Wayland compositor (or Hyprland with awww support)
- **systemd** - For service management (optional)
- **Rust** - For building from source

### Installing awww

```bash
# From AUR (if using Arch/Manjaro)
paru -S awww

# Or build from source
git clone https://codeberg.org/LGFae/awww
cd awww
cargo build --release
sudo install target/release/awww /usr/bin/
```

## Installation

### From crates.io (Recommended)

```bash
cargo install dms-awww
```

### From Source

1. **Clone this repository:**
   ```bash
   git clone https://github.com/yourusername/dms-awww-integration.git ~/code/dms-awww-integration
   cd ~/code/dms-awww-integration
   ```

2. **Build and install:**
   ```bash
   cargo install --path .
   ```

3. **Install the systemd service (optional):**
   ```bash
   cp systemd/dms-awww.service ~/.config/systemd/user/
   ```

4. **Edit the service if needed:**
   ```bash
   nano ~/.config/systemd/user/dms-awww.service
   ```

5. **Enable and start the service:**
   ```bash
   systemctl --user daemon-reload
   systemctl --user enable dms-awww.service
   systemctl --user start dms-awww.service
   ```

6. **Verify installation:**
   ```bash
   dms-awww --once
   ```

## Usage

Once installed and enabled, the service runs automatically:
- It starts when DMS starts
- It stops when DMS stops
- Changes wallpapers via awww when you change them in DMS
- Regenerates theme colors via DMS's matugen

### Command-line Options

```
dms-awww [OPTIONS]

Options:
  -c, --config <FILE>     Configuration file path
  -l, --log-level <LEVEL> Log level (trace, debug, info, warn, error)
  -o, --once              Run once and exit (for testing)
  -v, --verbose           Verbose output (debug level)
  -h, --help              Print help
  -V, --version           Print version
```

### Changing Wallpapers

Use DMS's normal wallpaper picker (typically via the right-click menu or DMS settings). The daemon will detect the change and apply it via awww.

## Configuration

The daemon can be configured via:

1. **Configuration file** - `~/.config/dms-awww/config.toml` or `~/.config/dms-awww/config.yaml`
2. **Environment variables** - Prefix with `DMS_AWWW_`
3. **Command-line arguments** - Override everything

### Example Configuration

```toml
# ~/.config/dms-awww/config.toml

[general]
log_level = "info"
log_file = "/tmp/dms_awww.log"
auto_detect_monitors = true
debounce_ms = 100

[dms]
session_file = "~/.local/state/DankMaterialShell/session.json"
settings_file = "~/.config/DankMaterialShell/settings.json"
cache_dir = "~/.cache/DankMaterialShell"

[niri]
# Optional: override auto-detection
# outputs = ["eDP-1", "HDMI-A-1"]

[awww]
enabled = true
# extra_args = []

[matugen]
enabled = true
default_scheme = "scheme-tonal-spot"
shell_dir = "/usr/share/quickshell/dms"
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `DMS_AWWW_LOG_LEVEL` | Log level (trace/debug/info/warn/error) |
| `DMS_AWWW_LOG_FILE` | Log file path |
| `DMS_AWWW_AUTO_DETECT_MONITORS` | Enable/disable auto-detection |
| `DMS_AWWW_SESSION_FILE` | Path to session.json |
| `DMS_AWWW_SETTINGS_FILE` | Path to settings.json |
| `DMS_AWWW_CACHE_DIR` | DMS cache directory |
| `DMS_AWWW_NIRI_OUTPUTS` | Comma-separated monitor names |
| `DMS_AWWW_AWWW_ENABLED` | Enable/disable awww |
| `DMS_AWWW_MATUGEN_ENABLED` | Enable/disable matugen |
| `DMS_AWWW_MATUGEN_SCHEME` | Default matugen scheme |
| `DMS_AWWW_SHELL_DIR` | Quickshell directory |

## Troubleshooting

### Service not running

```bash
# Check service status
systemctl --user status dms-awww.service

# View service logs
journalctl --user -u dms-awww.service -f

# Check watcher log
tail -f /tmp/dms_awww.log
```

### Wallpaper not changing

1. Verify awww is installed and working:
   ```bash
   which awww
   awww img -o HDMI-A-1 /path/to/image.jpg
   ```

2. Check if monitors are detected:
   ```bash
   niri msg outputs -j
   ```

3. Try running once to see detailed output:
   ```bash
   dms-awww --once -v
   ```

### Theme colors not updating

1. Verify DMS matugen is working:
   ```bash
   dms matugen queue --help
   ```

2. Check the logs for errors:
   ```bash
   journalctl --user -u dms-awww.service -n 50
   ```

## Performance

| Metric | Value |
|--------|-------|
| **Latency** | <10ms (event-driven) |
| **Idle CPU** | ~0% (no polling) |
| **Memory** | ~3-5MB RSS |
| **Binary size** | ~2-3MB (stripped) |
| **VRAM savings** | ~100-500MB |

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed information about the design, data flow, and implementation details.

## Migration from Bash Script

If you're migrating from the old bash script:

1. Install the Rust binary: `cargo install dms-awww`
2. Update your systemd service to use the new binary:
   ```ini
   ExecStart=%h/.cargo/bin/dms-awww
   ```
3. Remove the old script and service
4. No configuration changes needed - defaults match the old script behavior

## Contributing

Contributions are welcome! Please feel free to open issues or pull requests.

## License

MIT
