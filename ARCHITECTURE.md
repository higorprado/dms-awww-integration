# Architecture

## Background Problem

### DMS Wallpaper Storage

Dank Material Shell (DMS) stores wallpaper images in VRAM (Video RAM) for several reasons:

1. **Smooth transitions** - Crossfading between wallpapers requires both images in GPU memory
2. **Real-time effects** - Blur, dimming, and other effects are computed on the GPU
3. **Quick access** - No need to reload images from disk when switching

However, this approach has a significant downside:
- **VRAM consumption** - Each wallpaper at 4K resolution can use 25-50MB of VRAM
- **Limited slots** - DMS typically caches 5-10 wallpapers, using 100-500MB of VRAM
- **GPU overhead** - VRAM is a scarce resource on many GPUs

### The awww Alternative

The [awww](https://codeberg.org/LGFae/awww) tool takes a different approach:
- Writes wallpaper images directly to the Wayland compositor (Hyprland)
- No VRAM storage - the compositor manages the wallpaper
- Minimal GPU memory usage

```
┌─────────────────────────────────────────────────────────────┐
│                         DMS Approach                        │
├─────────────────────────────────────────────────────────────┤
│  DMS stores wallpapers in VRAM for transitions and effects  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │ Image 1 │  │ Image 2 │  │ Image 3 │  │ Image N │  VRAM   │
│  │  ~25MB  │  │  ~25MB  │  │  ~25MB  │  │  ~25MB  │  ═════  │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  100MB+ │
│       │            │            │            │              │
│       └────────────┴────────────┴────────────┘              │
│                            │                                │
│                            ▼                                │
│                     GPU (Hyprland)                          │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                         awww Approach                       │
├─────────────────────────────────────────────────────────────┤
│  awww writes directly to compositor - no VRAM caching       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Image sent to Hyprland →Displayed directly           │   │
│  └──────────────────────────────────────────────────────┘   │
│                            │                                │
│                            ▼                                │
│                     GPU (Hyprland)                          │
│                        Minimal VRAM                         │
└─────────────────────────────────────────────────────────────┘
```

## Solution Architecture

The solution integrates awww with DMS by monitoring DMS state and applying changes via awww:

```
┌──────────────────────────────────────────────────────────────────┐
│                        System Architecture                       │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐         ┌──────────────────────────────┐        │
│  │     DMS     │         │  DMS Wallpaper Watcher       │        │
│  │             │         │  (systemd service)           │        │
│  │ Wallpaper   │────────▶│                             │        │
│  │ Picker UI   │         │  1. Read session.json        │        │
│  └─────────────┘         │  2. Detect wallpaper change  │        │
│           │              │  3. Call awww                │        │
│           │              │  4. Trigger DMS matugen      │        │
│           ▼              └──────────┬───────────────────┘        │
│  ┌─────────────┐                      │                          │
│  │ session.json│                      ▼                          │
│  └─────────────┘         ┌──────────────────────┐                │
│                          │   awww (wallpaper)   │                │
│                          │   + matugen (theme)  │                │
│                          └──────────┬───────────┘                │
│                                     │                            │
│                                     ▼                            │
│                          ┌──────────────────────┐                │
│                          │     Hyprland         │                │
│                          │   (Wallpaper set)    │                │
│                          └──────────────────────┘                │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Watcher Script (`bin/dms-wallpaper-watcher`)

A bash script that runs in a continuous loop:

```bash
while true; do
    1. Read session.json
    2. Extract current wallpaper path
    3. If changed from last check:
       a. Call awww to set wallpaper
       b. Call dms matugen to update theme colors
    4. Sleep 1 second
done
```

**Key features:**
- Supports both single and per-monitor wallpapers
- Detects DMS dark/light mode for theme generation
- Reads matugen scheme type from DMS settings
- Comprehensive logging to `/tmp/dms_wallpaper_watcher.log`

### 2. systemd Service (`systemd/dms-wallpaper-watcher.service`)

A user systemd service that manages the watcher process:

```ini
[Unit]
Description=DMS Wallpaper Watcher
Requires=dms.service
After=dms.service
BindsTo=dms.service  # Stops when DMS stops

[Service]
Type=simple
ExecStart=~/.local/bin/dms-wallpaper-watcher
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
```

**Key features:**
- Auto-starts when DMS starts
- Auto-stops when DMS stops (BindsTo)
- Auto-restarts on failure
- Runs as user service (no root required)

## Data Flow

### Wallpaper Change Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Wallpaper Change Flow                    │
└─────────────────────────────────────────────────────────────┘

1. User changes wallpaper via DMS UI
                │
                ▼
2. DMS writes new wallpaper path to session.json
                │
                ▼
3. Watcher detects change (next poll, max 1 second delay)
                │
                ▼
4. Watcher calls awww
                │
                ├──▶ awww sends wallpaper to Hyprland
                │         │
                │         ▼
                │    Wallpaper displayed on screen
                │
                ▼
5. Watcher calls dms matugen queue
                │
                ├──▶ matugen generates color scheme
                │         │
                │         ▼
                │    GTK theme, dms-colors.json generated
                │
                ▼
6. Logged to /tmp/dms_wallpaper_watcher.log
```

### Session JSON Structure

The watcher reads `~/.local/state/DankMaterialShell/session.json`:

```json
{
  "wallpaperPath": "/path/to/wallpaper.jpg",
  "perMonitorWallpaper": false,
  "monitorWallpapers": {
    "HDMI-A-1": "/path/to/wallpaper1.jpg",
    "DP-1": "/path/to/wallpaper2.jpg"
  },
  "isLightMode": false
}
```

The watcher extracts the wallpaper path depending on whether per-monitor mode is enabled.

## Dependencies

### Runtime Dependencies

| Component | Purpose | Required By |
|-----------|---------|-------------|
| DMS | Shell environment | System |
| awww | Wallpaper rendering | Watcher |
| matugen | Theme generation | DMS |
| systemd | Service management | System |
| Hyprland | Wayland compositor | System |

### File Dependencies

| Path | Purpose | Access |
|------|---------|--------|
| `~/.local/state/DankMaterialShell/session.json` | Wallpaper state | Read |
| `~/.config/DankMaterialShell/settings.json` | Matugen settings | Read |
| `~/.cache/DankMaterialShell/` | Matugen cache | Write |
| `/tmp/dms_wallpaper_watcher.log` | Activity log | Write |

## Error Handling

The watcher handles several error conditions:

1. **File not found** - Wallpaper file doesn't exist → Warning logged
2. **Color values** - Solid colors (starting with #) → Ignored
3. **awww missing** - Error logged, wallpaper not set
4. **dms missing** - Warning logged, theme not updated

Each action is logged with timestamp and exit code for troubleshooting.

## Performance Considerations

- **Polling interval**: 1 second balance between responsiveness and CPU usage
- **Memory footprint**: ~2MB RSS for the watcher process
- **VRAM savings**: ~100-500MB compared to DMS wallpaper caching

## Future Enhancements

Possible improvements for future versions:

1. **Inotify-based watching** - Replace polling with filesystem events
2. **Multi-monitor support** - Different wallpapers per monitor via awww
3. **Configuration file** - Externalize settings (output name, log path)
4. **Better error recovery** - Retry logic for transient failures
