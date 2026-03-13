# LiveWall
## Product Requirements Document

**Set any video as a live wallpaper or screensaver on macOS Sonoma, Sequoia & Tahoe**

Version 1.4 · March 2026 · FOSS / MIT License

> **Changelog v1.4:** Final schema confirmed from real `entries.json`. Video injection strategy fully determined.

---

## 1. Overview

LiveWall is a free, open-source macOS utility that lets any user right-click a video file in Finder and instantly set it as their live wallpaper or screensaver. No technical knowledge is required. The entire flow from right-click to result completes in under 10 seconds.

**Target OS:** macOS Sonoma (14.x), Sequoia (15.x), and Tahoe (26.x). Intel + Apple Silicon.

**Important:** The aerial/video wallpaper system was introduced in macOS Sonoma. Ventura (13) and earlier do not have this system. LiveWall will detect pre-Sonoma versions and exit with a clear, friendly message.

**License:** MIT — fully open source, no telemetry, no cloud dependency.

---

## 2. Problem Statement

macOS provides no official public API for setting custom video wallpapers or screensavers. The built-in aerial system is a closed, daemon-managed pipeline with a fixed catalogue. Existing workarounds require manual terminal commands, admin passwords, and brittle hardcoded paths that break across macOS versions. LiveWall automates this entirely and uses runtime discovery to stay resilient across OS updates.

---

## 3. Core Design Principle — No Hardcoded Paths

This is the most important architectural constraint in the entire project.

> ⚠️ **LiveWall must NEVER hardcode any filesystem path** to Apple's wallpaper or screensaver storage. macOS has changed these paths between Sonoma, Sequoia, and Tahoe and will change them again. Any hardcoded path makes the app fragile and guarantees it will break for some users.

Instead, every filesystem path is resolved at runtime on each user's machine using the PathResolver module (Section 4.3). This means:

- The app works correctly on any macOS Sonoma, Sequoia, or Tahoe machine regardless of configuration.
- When Apple changes paths in a future OS update, only the PathResolver needs updating.
- Users with non-standard setups are handled gracefully.

---

## 4. Technical Architecture

### 4.1 How macOS Aerial Works (Context)

macOS Sonoma (14) introduced the aerial video wallpaper system. Sequoia (15) kept the same structure. Tahoe (26) moved to a fully user-mode path and dropped `idleassetsd`. Key actors confirmed by diagnostic on macOS 26.4:

| Actor | Present on | Role |
|---|---|---|
| `idleassetsd` | Sonoma (14), Sequoia (15) | Manages aerial assets. Requires admin. **NOT present on Tahoe.** |
| `WallpaperAgent` | Tahoe (26) — confirmed PID 1239 | Main orchestrator at `/System/Library/CoreServices/WallpaperAgent.app` |
| `WallpaperAerialsExtension` | Tahoe (26) — confirmed PID 1245 | Renders the video on screen. Note: **"Aerials" with an 's'** — not `WallpaperAerialExtension`. Lives at `/System/Library/ExtensionKit/Extensions/WallpaperAerialsExtension.appex` |
| `entries.json` | Tahoe: confirmed at `~/Library/.../aerials/manifest/entries.json` | JSON manifest registering all aerial assets. Note the `/manifest/` subdirectory. |

> ⚠️ The exact daemon name, folder paths, and manifest schema differ between macOS versions. The PathResolver (Section 4.3) handles this. Do NOT reference any specific path or process name directly outside of PathResolver.

---

### 4.2 Component Map

| Component | Technology | Responsibility |
|---|---|---|
| `livewallctl` | Rust | Core CLI: path resolution, video conversion, file ops, manifest patch, daemon restart |
| `path_resolver.rs` | Rust module | Runtime discovery of all required paths and daemon names — **NO hardcoded paths** |
| Finder Quick Actions | Automator `.workflow` (2 files) | Right-click menu entries — call `livewallctl` |
| Installer | Rust + shell | First-time setup: permissions, Quick Action install, ffmpeg unpack |
| ffmpeg | Bundled universal binary | MP4 → MOV conversion + PNG thumbnail extraction |
| LiveWall.app | Swift (optional, v1.1) | Menu-bar wrapper for status and uninstall |

---

### 4.3 PathResolver — Runtime Path Discovery

PathResolver is the most critical module in the codebase. It is a Rust struct (`cli/src/path_resolver.rs`) that resolves all required filesystem paths and daemon names at runtime using a layered strategy. Called once at startup; result passed to all other modules.

#### 4.3.1 Output: ResolvedPaths

```rust
pub struct ResolvedPaths {
    pub videos_dir: PathBuf,         // Where .mov files are stored
    pub manifest_path: PathBuf,      // Full path to entries.json
    pub thumbnails_dir: PathBuf,     // Where PNG thumbnails go
    pub renderer_process: String,    // WallpaperAerialsExtension or equiv
    pub agent_process: String,       // WallpaperAgent or idleassetsd
    pub requires_elevation: bool,    // true on Sonoma/Sequoia, false on Tahoe
    pub macos_version: MacOSVersion, // Tahoe | Sequoia | Sonoma | Unsupported
}
```

#### 4.3.2 Discovery Strategy (in order)

**Strategy 0 — Version gate. Fail fast if unsupported.**

Before any path discovery, check the OS version via `sw_vers`. If Ventura (13) or earlier, exit immediately:

```
Error: LiveWall requires macOS Sonoma (14) or later.
Video wallpapers were introduced in macOS Sonoma.
Your Mac is running macOS [version]. Please upgrade to use LiveWall.
```

**Strategy 1 — Ask the running wallpaper process via `lsof`.**

Confirmed process names from diagnostic on macOS 26.4:

```rust
// Tahoe:          WallpaperAerialsExtension  (Aerials with an 's')
// Sonoma/Sequoia: WallpaperAerialExtension   (no 's') + idleassetsd
let pid = pgrep(["WallpaperAerialsExtension", "WallpaperAerialExtension", "idleassetsd"]);
let open_files = lsof("-p", pid);
let mov_path = open_files.find(|f| f.ends_with(".mov"));
return mov_path.parent(); // videos directory
```

**Strategy 2 — Probe known candidate paths by OS version priority.**

| Priority | Videos Path | Manifest Path | macOS | Admin? |
|---|---|---|---|---|
| 1 ✅ CONFIRMED | `~/Library/Application Support/com.apple.wallpaper/aerials/videos/` | `~/Library/Application Support/com.apple.wallpaper/aerials/manifest/entries.json` | Tahoe (26) | No |
| 2 | `~/Library/Containers/com.apple.wallpaper.agent/.../extension-com.apple.wallpaper.extension.aerials/` | Check container for `entries.json` | Tahoe fallback | No |
| 3 | `/Library/Application Support/com.apple.idleassetsd/Customer/4KSDR240FPS/` | `/Library/Application Support/com.apple.idleassetsd/Customer/entries.json` | Sonoma/Sequoia | Yes |
| 4 | `~/Library/Application Support/com.apple.idleassetsd/Customer/4KSDR240FPS/` | `~/Library/Application Support/com.apple.idleassetsd/Customer/entries.json` | Sonoma/Sequoia user-mode | No |
| 5 | `/Library/Application Support/com.apple.idleassetsd/Customer/2KSDR240FPS/` | Same `entries.json` as above | Sonoma/Sequoia 2K | Yes |

**Strategy 3 — Search `~/Library` and `/Library` for UUID-named `.mov` files.**

```
// Find dirs containing files matching: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.mov
```

**Strategy 4 — Fail with diagnostic error.**

```rust
PathResolverError::NotFound {
    message: "LiveWall could not locate wallpaper storage.",
    diagnostic_cmd: "ls ~/Library/Application\\ Support/com.apple*",
    issue_url: "github.com/[owner]/livewall/issues/new",
}
```

#### 4.3.3 Daemon Resolution

Two processes need to be signalled — the renderer (immediate reload) and the agent (persist change). Both confirmed on Tahoe.

| Process | macOS | Confirmed Binary Path | Signal |
|---|---|---|---|
| `WallpaperAerialsExtension` | Tahoe | `/System/Library/ExtensionKit/Extensions/WallpaperAerialsExtension.appex/Contents/MacOS/WallpaperAerialsExtension` | SIGTERM — launchd auto-restarts |
| `WallpaperAgent` | Tahoe | `/System/Library/CoreServices/WallpaperAgent.app/Contents/MacOS/WallpaperAgent` | SIGTERM — launchd auto-restarts |
| `idleassetsd` | Sonoma/Sequoia | `/System/Library/PrivateFrameworks/TVIdleServices.framework/.../idleassetsd` | SIGTERM — launchd auto-restarts |

Send SIGTERM to the renderer process first, wait 1s, then SIGTERM the agent. Both are launchd-managed and will restart automatically. Wait up to 3s and verify both PIDs are live again before reporting success.

#### 4.3.4 Caching

PathResolver results are cached in memory for the lifetime of the process. Results must **NOT** be persisted to disk — paths can change between macOS updates and must be re-resolved fresh on each run.

---

### 4.4 Video Injection Strategy

> **This is the most important architectural decision in the project**, determined by analysing the real `entries.json` schema.

#### 4.4.1 Key Finding: entries.json URLs Always Point to Apple's CDN

Confirmed from real data: every entry in `entries.json` — including entries whose `.mov` files are locally cached — uses an `https://sylvan.apple.com/...` URL in the `url-4K-SDR-240FPS` field. macOS downloads the video from the CDN and caches it as `<UUID>.mov` in the videos directory. The manifest URL is never updated to a local path.

This means we cannot add a custom entry with a local file path and expect macOS to play it using the normal manifest URL flow — unless it accepts `file://` URLs (untested, see below).

#### 4.4.2 Two Viable Approaches — Builder Must Test Both

**Approach A — `file://` URL injection (preferred if it works)**

Add a new entry to `entries.json` with a fresh UUID and set `url-4K-SDR-240FPS` to a `file://` URL. Copy the video to `aerials/videos/<UUID>.mov`. If macOS honours `file://` URLs, this is the cleanest approach — no Apple aerials are modified.

```json
"url-4K-SDR-240FPS": "file:///Users/<user>/Library/.../aerials/videos/<UUID>.mov"
```

**Approach B — File swap (confirmed working, used by YouTube tutorial)**

Pick an existing Apple aerial UUID from `entries.json`. Rename the user's video to `<UUID>.mov`. Replace the cached file at `aerials/videos/<UUID>.mov`. The manifest entry already exists — macOS plays the local file since it's cached under that UUID. Restart `WallpaperAerialsExtension`.

> ⚠️ **Approach B is destructive** — it overwrites one of the user's downloaded Apple aerials. LiveWall MUST save a backup of the original `.mov` before overwriting, and offer `livewallctl restore` to put it back.

**Approach C — Fallback if both A and B fail**

Add the entry to `entries.json` AND copy the file, then open System Settings → Wallpaper to force a UI refresh, prompting the user to select the new entry manually.

**Implementation order:** Try A first. If the video does not appear in System Settings within 5 seconds of daemon restart, fall back to B. Document findings in the repo README.

---

### 4.5 set-wallpaper / set-screensaver Pipeline

Both commands execute the same pipeline. The only difference is which System Settings panel is opened at the end.

1. Call `PathResolver::resolve()`. Fail fast with a clear error if it returns `Err`.
2. Validate the input file exists and is `.mp4` or `.mov`.
3. If `.mp4`: invoke bundled ffmpeg to transcode to `.mov` (H.264, no audio, loop-safe). Write to a temp path.
4. Generate a UUIDv4 for this asset.
5. Extract a PNG thumbnail at timestamp `00:00:01` using ffmpeg. Save as `<UUID>.png` to `resolved.thumbnails_dir/`. **PNG format confirmed — NOT JPEG.**
6. Copy the `.mov` to `resolved.videos_dir/<UUID>.mov`.
7. Read `resolved.manifest_path`. Create a timestamped `.bak` backup. Append new asset entry (Section 4.6). Validate JSON. Write back.
8. Send SIGTERM to `WallpaperAerialsExtension` first, wait 1s, then SIGTERM `WallpaperAgent` (Tahoe) or `idleassetsd` (Sonoma/Sequoia). Wait up to 3s for both to reappear.
9. Verify the video is accessible. If not visible after 5s, fall back to Approach B (Section 4.4.2).
10. Print success. Open System Settings → Wallpaper: `open 'x-apple.systempreferences:com.apple.preference.desktopscreeneffect'`

---

### 4.6 Manifest Entry Schema — Fully Confirmed

All field names confirmed from real `entries.json` on macOS 26.4. Every key below uses the **exact name found in production**.

```json
{
  "accessibilityLabel": "<user name or filename>",
  "categories": [],
  "id": "<UUIDv4>",
  "includeInShuffle": true,
  "localizedNameKey": "LIVEWALL_<UUID>",
  "pointsOfInterest": {},
  "preferredOrder": 99999,
  "previewImage": "<UUID>.png",
  "shotID": "LW_<short-id>",
  "showInTopLevel": true,
  "subcategories": [],
  "url-4K-SDR-240FPS": "file://<absolute-path-to-UUID.mov>",
  "source": "livewall"
}
```

> ⚠️ **CRITICAL:** The video URL field key is `url-4K-SDR-240FPS` **with hyphens** — NOT `url4KSDR240FPS`. Using the wrong key name will silently fail.

> ⚠️ The `"source": "livewall"` field is non-standard and ignored by macOS. It exists purely so `livewallctl list` and `livewallctl remove` can identify user-added entries. Do not remove it.

---

### 4.7 Permissions Model

| Scenario | Path location | Admin required? | Mechanism |
|---|---|---|---|
| Tahoe (confirmed) | `~/Library` — user-writable | **No** | No elevation needed at all |
| Sonoma/Sequoia primary | `/Library` — system | **Yes** | macOS Authorization Services — cache credential after first use, never prompt again |
| Sonoma/Sequoia user-mode | `~/Library` — user-writable | **No** | No elevation needed |

Never use `sudo` directly. Use `AuthorizationExecuteWithPrivileges` or `SMJobBless` privileged helper.

---

## 5. Repository Structure

```
livewall/
├── cli/                        # Rust crate: livewallctl binary
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── set.rs          # set-wallpaper + set-screensaver
│   │   │   ├── list.rs
│   │   │   ├── remove.rs
│   │   │   └── install.rs
│   │   ├── path_resolver.rs    # *** RUNTIME PATH DISCOVERY — NO HARDCODED PATHS ***
│   │   ├── ffmpeg.rs           # ffmpeg invocation helpers
│   │   ├── manifest.rs         # entries.json read/write/validate/backup
│   │   ├── daemon.rs           # daemon restart logic
│   │   └── permissions.rs      # macOS Authorization Services
│   └── Cargo.toml
├── quick-actions/
│   ├── SetAsLiveWallpaper.workflow
│   └── SetAsScreensaver.workflow
├── installer/
├── vendor/ffmpeg               # Universal binary (arm64 + x86_64)
├── tests/
│   └── path_resolver_tests.rs  # Unit tests for each discovery strategy
├── README.md
└── LICENSE                     # MIT
```

---

## 6. Finder Quick Actions

Two Automator `.workflow` files installed into `~/Library/Services/`. Each passes the selected file path to `livewallctl`.

### 6.1 SetAsLiveWallpaper.workflow

- Type: Quick Action
- Workflow receives: files in Finder
- Input file types: `com.apple.quicktime-movie`, `public.mpeg-4`
- Shell: `/bin/bash`

```bash
for f in "$@"; do
  /usr/local/bin/livewallctl set-wallpaper "$f"
done
```

### 6.2 SetAsScreensaver.workflow

Identical to 6.1 but calls `set-screensaver` subcommand.

### 6.3 Installation

The installer copies both `.workflow` bundles to `~/Library/Services/` then runs:

```bash
/System/Library/CoreServices/pbs -update
```

This flushes Finder's Quick Actions cache. The installer must also verify Quick Actions are visible in Finder and prompt the user to enable them if not.

---

## 7. Dependencies

| Dependency | Version | License | Usage |
|---|---|---|---|
| Rust | 1.77+ | MIT/Apache | Core CLI language |
| clap | 4.x | MIT | CLI argument parsing |
| serde + serde_json | 1.x | MIT | `entries.json` parsing |
| uuid | 1.x | MIT/Apache | Asset UUID generation |
| tokio | 1.x | MIT | Async process management |
| which | 6.x | MIT | Locate bundled ffmpeg binary |
| ffmpeg (static binary) | 6.x | LGPL | Video conversion + PNG thumbnail |
| Automator | macOS built-in | — | Quick Action `.workflow` files |

ffmpeg must be bundled as a universal binary (arm64 + x86_64). No system ffmpeg assumed or required.

---

## 8. Known Limitations & Mitigations

| Issue | Cause | Mitigation |
|---|---|---|
| Desktop may go black after login following wallpaper change | macOS aerial system limitation | Document in README; show one-time in-app notice |
| Live wallpaper may not re-activate after lock/unlock without restart | Daemon caching behaviour | Document; v2 to investigate re-signal approach |
| Manifest schema may change in a future macOS update | Apple internal API — no public contract | PathResolver version-checks at runtime; fails with actionable error and GitHub issue link |
| Approach B overwrites an Apple aerial | File swap mechanism | Always back up original `.mov`; `livewallctl restore` command included |
| Admin password required on Sonoma/Sequoia `/Library` path | PathResolver output dependent | Authorization Services caches credential after first use — one-time only |

---

## 9. Acceptance Criteria

The build is complete when all of the following pass on clean installs of **Sonoma, Sequoia, AND Tahoe** on both Apple Silicon and Intel:

- [ ] Right-clicking a `.mp4` file in Finder shows "Set as Live Wallpaper" and "Set as Screensaver" in Quick Actions.
- [ ] Both commands complete in under 10 seconds with no terminal window visible to the user.
- [ ] The video appears in System Settings → Wallpaper without any manual steps.
- [ ] The video appears in System Settings → Screen Saver without any manual steps.
- [ ] `livewallctl list` prints all previously added custom videos with UUIDs and filenames.
- [ ] `livewallctl remove <id>` removes the entry from the manifest and deletes the `.mov` and `.png` files cleanly.
- [ ] The manifest is never left corrupted — a `.bak` file is always created before any mutation.
- [ ] If PathResolver cannot find the wallpaper storage location, CLI exits with a clear human-readable error and a GitHub issue link.
- [ ] No crash or hang if the wallpaper daemon is not running at invocation time.
- [ ] No network requests are made during normal operation.
- [ ] PathResolver unit tests pass for each of the 3 discovery strategies independently using a mocked filesystem.

---

## 10. Out of Scope (Future Versions)

- GUI preferences window (v2).
- Multi-monitor per-display video assignment (v2).
- GIF or shader/WebGL wallpaper support (v2).
- Ventura (13) and earlier — aerial system does not exist. LiveWall exits gracefully with an explanation.
- App Store distribution — sandboxing is incompatible with the required file access model.
- Screensaver overlay widgets (clock, text).

---

## 11. Open Questions for Builder

All major unknowns are resolved from real diagnostic data on macOS 26.4. Two items remain that can only be answered during implementation:

**Confirmed ✅**

- ✅ Videos path: `~/Library/Application Support/com.apple.wallpaper/aerials/videos/`
- ✅ Manifest path: `~/Library/Application Support/com.apple.wallpaper/aerials/manifest/entries.json`
- ✅ All exact JSON field names from real `entries.json`: `accessibilityLabel`, `categories`, `id`, `includeInShuffle`, `localizedNameKey`, `pointsOfInterest`, `preferredOrder`, `previewImage`, `shotID`, `showInTopLevel`, `subcategories`, `url-4K-SDR-240FPS`
- ✅ `url-4K-SDR-240FPS` uses **hyphens** — NOT camelCase
- ✅ Locally cached videos keep CDN URLs in manifest; macOS matches by UUID
- ✅ Thumbnails are PNG in `aerials/thumbnails/` — NOT JPEG
- ✅ Daemon targets: `WallpaperAerialsExtension` + `WallpaperAgent` on Tahoe
- ✅ No admin required on Tahoe — all paths under `~/Library`
- ✅ `idleassetsd` does not exist on Tahoe

**Still unknown ⚠️**

- ⚠️ **Does macOS honour `file://` URLs in `url-4K-SDR-240FPS`?** (Approach A). This is the first thing to test during implementation. If yes → use Approach A. If no → use Approach B with backup. Document result in README.
- ⚠️ **Sonoma/Sequoia schema still needs verification.** Run the diagnostic script on a Sonoma (14) or Sequoia (15) machine to confirm `idleassetsd` paths and `entries.json` field names on those versions.

---

## 12. Reference Projects

| Project | URL | Relevance |
|---|---|---|
| FalconLee1011/Customized-Aerial-Screen-Saver | github.com/FalconLee1011/Customized-Aerial-Screen-Saver | Closest existing tool. Source of truth for manifest schema. |
| AerialScreensaver/Aerial | github.com/AerialScreensaver/Aerial | Mature FOSS screensaver with Tahoe support. Reference for daemon interaction. |
| mikeswanson/WallGet | github.com/mikeswanson/WallGet | Shows `idleassetsd` folder layout and user-mode vs legacy-mode path detection. |
| Screensaver Spelunking (footle.org) | footle.org/2025/03/21/screensaver-spelunking/ | Deep dive into `WallpaperVideoExtension`, `entries.json`, and lsof approach. |
