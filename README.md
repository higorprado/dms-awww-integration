# dms-awww-integration

Efficient wallpaper management for [Dank Material Shell (DMS)](https://github.com/AvengeMedia/DankMaterialShell) using [awww](https://codeberg.org/LGFae/awww).

## Problem

DMS stores wallpapers in VRAM for quick transitions and effects. While this provides smooth animations, it consumes significant GPU memory (VRAM). The [awww](https://codeberg.org/LGFae/awww) tool provides a more memory-efficient alternative for setting wallpapers by writing directly to the Wayland compositor (Hyprland) without storing images in VRAM.

## Solution

This project provides a watcher service that:
1. Monitors DMS's `session.json` for wallpaper changes
2. Applies the new wallpaper via `awww` (efficient, no VRAM usage)
3. Triggers DMS's matugen to regenerate theme colors to keep everything in sync

This gives you the best of both worlds: DMS's UI and theming with awww's efficient wallpaper rendering.

## Prerequisites

- **Dank Material Shell (DMS)** - Installed and configured ([Github](https://github.com/AvengeMedia/DankMaterialShell))
- **awww** - Wallpaper utility for Hyprland ([Codeberg](https://codeberg.org/LGFae/awww))
- **matugen** - Material Design color generator (typically comes with DMS)
- **systemd** - For service management

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

1. **Clone this repository:**
   ```bash
   git clone https://github.com/yourusername/dms-awww-integration.git ~/code/dms-awww-integration
   cd ~/code/dms-awww-integration
   ```

2. **Install the watcher script:**
   ```bash
   cp bin/dms-wallpaper-watcher ~/.local/bin/
   chmod +x ~/.local/bin/dms-wallpaper-watcher
   ```

3. **Install the systemd service:**
   ```bash
   cp systemd/dms-wallpaper-watcher.service ~/.config/systemd/user/
   ```

4. **Edit the service if needed** (e.g., different username):
   ```bash
   nano ~/.config/systemd/user/dms-wallpaper-watcher.service
   ```

5. **Enable and start the service:**
   ```bash
   systemctl --user daemon-reload
   systemctl --user enable dms-wallpaper-watcher.service
   systemctl --user start dms-wallpaper-watcher.service
   ```

6. **Verify installation:**
   ```bash
   ./test.sh
   ```

## Usage

Once installed and enabled, the service runs automatically:
- It starts when DMS starts
- It stops when DMS stops
- Changes wallpapers via awww when you change them in DMS
- Regenerates theme colors via DMS's matugen

### Changing Wallpapers

Use DMS's normal wallpaper picker (typically via the right-click menu or DMS settings). The watcher will detect the change and apply it via awww.

## Configuration

The watcher script has a few configurable variables at the top:

```bash
SESSION_FILE="$HOME/.local/state/DankMaterialShell/session.json"
LOG_FILE="/tmp/dms_wallpaper_watcher.log"
AWWW_OUTPUT="HDMI-A-1"  # Your monitor output name
```

To find your monitor output name:
```bash
hyprctl monitors | grep "Monitor"
```

## Troubleshooting

### Service not running

```bash
# Check service status
systemctl --user status dms-wallpaper-watcher.service

# View service logs
journalctl --user -u dms-wallpaper-watcher.service -f

# Check watcher log
tail -f /tmp/dms_wallpaper_watcher.log
```

### Wallpaper not changing

1. Verify awww is installed and working:
   ```bash
   which awww
   awww img -o HDMI-A-1 /path/to/image.jpg
   ```

2. Check the correct output name for your monitor:
   ```bash
   hyprctl monitors
   ```

3. Update `AWWW_OUTPUT` in the watcher script if needed.

### Theme colors not updating

1. Verify DMS matugen is working:
   ```bash
   dms matugen queue --help
   ```

2. Check the watcher log for errors:
   ```bash
   tail -f /tmp/dms_wallpaper_watcher.log
   ```

## File Structure

```
dms-awww-integration/
├── bin/
│   └── dms-wallpaper-watcher    # Main watcher script
├── systemd/
│   └── dms-wallpaper-watcher.service  # systemd user unit
├── README.md                    # This file
├── ARCHITECTURE.md              # Detailed design documentation
└── test.sh                      # Installation verification script
```

## How It Works

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed information about the problem, solution, and design decisions.

## License

MIT

## Contributing

Contributions are welcome! Feel free to open issues or pull requests.
