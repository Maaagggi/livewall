#  LiveWall

> [!CAUTION]
> **UNDER CONSTRUCTION**: This project is currently in early development. Some features, particularly Finder Quick Actions on macOS Sequoia, are still being refined and may not function as expected.

LiveWall is a powerful macOS utility that allows you to set custom videos as live wallpapers and screensavers, mimicking the native behavior of Apple's aerial wallpapers.

## 🚀 Features

- **Custom Video Support**: Set any `.mp4` or `.mov` as your wallpaper.
- **Dynamic Path Resolution**: No hardcoded paths—resilient to macOS updates (Sonoma, Sequoia, Tahoe).
- **Finder Integration**: Right-click any video to set it as a wallpaper or screensaver (In Progress).
- **Native Experience**: Uses macOS system signals to refresh wallpapers instantly without reboots.

## 🛠️ Installation

### Prerequisites

- **macOS Sonoma (14.0)** or later.
- **Rust** (if building from source).
- **ffmpeg** (required for video processing).

### Building from Source

```bash
cd cli
cargo build --release
sudo ./target/release/livewallctl install
```

## 📖 Usage

### Command Line

```bash
# Set a video as wallpaper
livewallctl set-wallpaper path/to/video.mp4

# List user-added wallpapers
livewallctl list

# Remove a wallpaper by ID
livewallctl remove <UUID>
```

### Finder Quick Actions (Beta)

1. Right-click a video file in Finder.
2. Select **Quick Actions** > **Set as Live Wallpaper**.

*Note: If Quick Actions are not visible or fail to run, please consult the [walkthrough](https://github.com/Maaagggi/livewall/blob/main/quick_actions_debug_summary.md) for troubleshooting steps.*

## 🏗️ Technical Architecture

The core of LiveWall is a Rust-based CLI (`livewallctl`) that manages:
1. **Path Resolution**: Discovers the local Apple Aerials database and storage paths.
2. **Manifest Management**: Safely patches `entries.json` with atomicity and backups.
3. **Ffmpeg Pipeline**: Handles video transcoding and thumbnail extraction.
4. **Daemon Signaling**: Signals `WallpaperAgent` and `idleassetsd` to apply changes.

## 📄 License

This project is licensed under the MIT License.
