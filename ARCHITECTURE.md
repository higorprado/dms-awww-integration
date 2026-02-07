# Architecture

## Overview

Dank Material Shell (DMS) caches wallpapers in VRAM for smooth transitions, consuming 100-500MB. This daemon replaces that with the [awww](https://codeberg.org/LGFae/awww) tool, which renders wallpapers directly through the Wayland compositor.

**Result:** Same visual experience with minimal VRAM usage.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        dms-awww Daemon                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────┐   │
│  │   Config     │    │   Inotify    │    │     Executor         │   │
│  │   Manager    │    │   Watcher    │    │   (awww + matugen)   │   │
│  └──────────────┘    └──────┬───────┘    └──────────┬───────────┘   │
│                              │                       │              │
│                              ▼                       ▼              │
│                       ┌─────────────┐        ┌─────────────┐        │
│                       │  DMS JSON   │        │   Niri      │        │
│                       │   Parser    │        │   IPC       │        │
│                       └─────────────┘        └─────────────┘        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │  Wayland Compositor │
                    │     (Niri/Hyprland) │
                    └─────────────────────┘
```

## Modules

| Module | Purpose |
|--------|---------|
| `main.rs` | Entry point, event loop, CLI handling |
| `config` | Configuration loading (defaults → files → env → CLI) |
| `dms` | DMS session.json parsing |
| `watcher` | Inotify file monitoring with debouncing |
| `niri` | Monitor auto-detection via `niri msg outputs` |
| `executor` | Sequential wallpaper + theme application |
| `error` | Error types with thiserror |

## Data Flow

```
┌──────────────────────────────────────────────────────────────────────┐
│                     Wallpaper Change Flow                            │
└──────────────────────────────────────────────────────────────────────┘

User changes wallpaper in DMS
            │
            ▼
DMS writes session.json
            │
            ▼
Inotify detects change (<10ms)
            │
            ▼
Parse JSON → Detect wallpaper change
            │
            ├──▶ awww (parallel per monitor) → Wallpaper displayed
            │                │
            │                ▼ (wait for completion)
            │                │
            └──▶ matugen → Theme colors updated
```

**Key design decisions:**
- **Event-driven:** inotify, not polling (zero idle CPU)
- **Sequential awww → matugen:** Prevents visual flicker
- **Parallel per monitor:** Multiple awww instances run concurrently

## Performance

| Metric | Bash Script | Rust Daemon | Improvement |
|--------|-------------|-------------|-------------|
| Latency | 0-1s (polling) | <10ms (event) | ~100x |
| Idle CPU | Wake every 1s | Zero | ~100% |
| Memory | ~2-5MB | ~5MB | Comparable |
| VRAM Savings | 100-500MB | Same | — |

## Configuration

**File locations** (checked in order):
- `~/.config/dms-awww/config.toml`
- `~/.config/dms-awww/config.yaml`
- `/etc/dms-awww/config.toml`

**Environment variables:** `DMS_AWWW_*` prefix (e.g., `DMS_AWWW_LOG_LEVEL`)

**Precedence:** defaults → files → env → CLI

## Dependencies

| Component | Required |
|-----------|----------|
| DMS | Yes |
| awww | Yes |
| matugen | Yes (for theming) |
| Niri | Yes (for auto-detection) |
| systemd | Optional (service management) |
