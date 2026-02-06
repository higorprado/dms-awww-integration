# Architecture

## Overview

dms-awww is a Rust daemon that provides efficient wallpaper management for Dank Material Shell (DMS) by integrating with the awww wallpaper tool. This document describes the architecture, design decisions, and implementation details.

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
- Writes wallpaper images directly to the Wayland compositor (Niri/Hyprland)
- No VRAM storage - the compositor manages the wallpaper
- Minimal GPU memory usage

## Solution Architecture

The Rust daemon uses an event-driven architecture with inotify for instant detection of wallpaper changes:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Rust Daemon (dms-awww)                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────┐   │
│  │   Config     │    │   Inotify    │    │    Task Scheduler    │   │
│  │   Manager    │    │   Watcher    │    │    (async tokio)     │   │
│  └──────┬───────┘    └──────┬───────┘    └──────────┬───────────┘   │
│         │                   │                        │               │
│         ▼                   ▼                        ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────┐   │
│  │    JSON      │    │   Change     │    │   Parallel Workers   │   │
│  │   Parser     │    │  Detector    │    │                      │   │
│  └──────────────┘    └──────┬───────┘    └──────────────────────┘   │
│                              │                                        │
│                              ▼                                        │
│                       ┌─────────────┐                                │
│                       │  Dispatch   │                                │
│                       │   Logic     │                                │
│                       └──────┬──────┘                                │
│                              │                                        │
│              ┌───────────────┴───────────────┐                       │
│              ▼                               ▼                       │
│      ┌───────────────┐               ┌──────────────┐               │
│      │ awww worker   │               │ matugen      │               │
│      │ (async task)  │               │ worker       │               │
│      └───────────────┘               └──────────────┘               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Components

### Component Overview

| Component | Purpose | Technology |
|-----------|---------|------------|
| Config Manager | Load/validate configuration | serde, config-rs |
| Inotify Watcher | Monitor session.json for changes | notify crate |
| JSON Parser | Parse DMS session/settings JSON | serde_json |
| Monitor Detector | Auto-detect Niri outputs | `niri msg outputs` |
| Task Scheduler | Async task orchestration | tokio |
| Worker Pool | Parallel command execution | tokio::task |
| Logger | Structured logging | tracing |

### Module Structure

```
src/
├── main.rs          # Main entry point, event loop
├── error.rs         # Error types with thiserror
├── config/
│   └── mod.rs       # Configuration loading and validation
├── dms/
│   └── mod.rs       # DMS JSON parsing
├── watcher/
│   └── mod.rs       # Inotify file watching
├── niri/
│   └── mod.rs       # Niri IPC integration
└── executor/
    └── mod.rs       # Command execution
```

## Component Details

### 1. Configuration System (`src/config/`)

The configuration system supports multiple sources with precedence:

1. **Default values** - Built-in sensible defaults
2. **Config files** - TOML/YAML in `~/.config/dms-awww/` or `/etc/dms-awww/`
3. **Environment variables** - `DMS_AWWW_*` prefix
4. **Command-line args** - Highest priority

**Key features:**
- Path expansion for `~` and `$VAR`
- Validation of required commands and paths
- Per-feature enable/disable flags

### 2. Inotify Watcher (`src/watcher/`)

Replaces the bash script's polling with event-driven monitoring:

```rust
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<FileEvent>,
    path: PathBuf,
}
```

**Benefits:**
- Near-instant detection (<10ms vs 1s polling)
- Zero CPU usage when idle
- Better battery life
- Debouncing support for rapid changes

### 3. DMS JSON Parser (`src/dms/`)

Proper JSON parsing using serde:

```rust
#[derive(Deserialize)]
pub struct SessionJson {
    pub wallpaper_path: Option<String>,
    pub per_monitor_wallpaper: Option<bool>,
    pub monitor_wallpapers: HashMap<String, String>,
    pub is_light_mode: Option<bool>,
}
```

**Handles:**
- Single wallpaper mode
- Per-monitor wallpapers
- Light/dark mode detection

### 4. Niri Monitor Detection (`src/niri/`)

Auto-detects monitors using Niri's IPC:

```rust
pub async fn detect_outputs() -> Result<Vec<String>> {
    let output = Command::new("niri")
        .args(["msg", "outputs", "-j"])
        .output()
        .await?;
    // Parse JSON and extract enabled output names
}
```

**Features:**
- JSON parsing of monitor info
- Filters for enabled outputs only
- Fallback for manual configuration

### 5. Parallel Executor (`src/executor/`)

Runs awww and matugen concurrently:

```rust
pub async fn apply_wallpaper(&self, change: &WallpaperChange) -> Result<()> {
    let (awww_result, matugen_result) = tokio::join!(
        self.apply_awww(change),
        self.apply_matugen(change)
    );
    // Handle results
}
```

**Benefits:**
- Faster total execution time
- Independent error handling
- Per-monitor parallel processing

### 6. Error Handling (`src/error.rs`)

Comprehensive error types using thiserror:

```rust
#[derive(Debug, Error)]
pub enum DmsAwwwError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    // ... more variants
}
```

**Features:**
- Source tracking for errors
- User-friendly error messages
- Distinguish recoverable vs critical errors

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
3. Inotify detects file change (<10ms)
                │
                ▼
4. Watcher debounces rapid changes
                │
                ▼
5. Daemon parses session.json
                │
                ▼
6. Detects wallpaper change
                │
                ├──▶ 7a. Spawn awww task (parallel)
                │         │
                │         ▼
                │    Wallpaper displayed on screen
                │
                └──▶ 7b. Spawn matugen task (parallel)
                          │
                          ▼
                     Theme colors updated
                │
                ▼
8. Both tasks complete, logging results
```

### Session JSON Structure

The daemon reads `~/.local/state/DankMaterialShell/session.json`:

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

The daemon extracts the wallpaper path depending on whether per-monitor mode is enabled.

## Configuration

### Configuration File Locations

Checked in order:
1. `~/.config/dms-awww/config.toml`
2. `~/.config/dms-awww/config.yaml`
3. `~/.config/dms-awww/config.yml`
4. `/etc/dms-awww/config.toml`

### Environment Variables

All configuration can be overridden via environment variables with the `DMS_AWWW_` prefix:

- `DMS_AWWW_LOG_LEVEL` - Log level
- `DMS_AWWW_SESSION_FILE` - Path to session.json
- `DMS_AWWW_NIRI_OUTPUTS` - Comma-separated monitor list
- And more...

## Performance

### Comparison: Bash vs Rust

| Metric | Bash Script | Rust Daemon | Improvement |
|--------|-------------|-------------|-------------|
| **Latency** | 0-1s (polling) | <10ms (event) | ~100x faster |
| **Idle CPU** | Wake every 1s | Zero (event) | ~100% reduction |
| **Code Size** | ~100 lines | ~800-1000 lines | More features |
| **Reliability** | Fragile parsing | Proper JSON | Significantly better |
| **Configurability** | Hardcoded | File + env | Much more flexible |
| **Binary Size** | N/A (script) | ~2-3MB | Single binary |
| **Memory** | ~2MB RSS | ~3-5MB | Minimal increase |
| **Multi-monitor** | Manual | Auto-detect | Automatic |
| **VRAM Savings** | 100-500MB | Same | Maintained |

## Dependencies

### Runtime Dependencies

| Component | Purpose | Required By |
|-----------|---------|-------------|
| DMS | Shell environment | System |
| awww | Wallpaper rendering | Daemon |
| matugen | Theme generation | DMS |
| systemd | Service management | System (optional) |
| Niri | Wayland compositor | System |

### File Dependencies

| Path | Purpose | Access |
|------|---------|--------|
| `~/.local/state/DankMaterialShell/session.json` | Wallpaper state | Read |
| `~/.config/DankMaterialShell/settings.json` | Matugen settings | Read |
| `~/.cache/DankMaterialShell/` | Matugen cache | Write |
| `/tmp/dms_awww.log` | Activity log | Write |

## Error Handling

The daemon handles several error conditions:

1. **File not found** - Wallpaper file doesn't exist → Error logged
2. **Color values** - Solid colors (starting with #) → Ignored
3. **awww missing** - Error logged, startup fails
4. **dms missing** - Error logged, startup fails
5. **Missing session file** - Warning logged, waits for file

Each action is logged with tracing for troubleshooting.

## Future Enhancements

Possible improvements for future versions:

1. **Hyprland support** - Add Hyprland IPC integration
2. **More wallpaper tools** - Support for swww, waypaper
3. **Custom commands** - User-defined hooks on wallpaper change
4. **DBus interface** - Query/control the daemon
5. **Wallpaper history** - Keep history of recent wallpapers
6. **Transitional effects** - Support for crossfades

## Testing

The project includes:

1. **Unit tests** - For each module (config, parsing, etc.)
2. **Integration tests** - End-to-end scenarios
3. **Manual testing** - Using `--once` flag

Run tests with:
```bash
cargo test
```
