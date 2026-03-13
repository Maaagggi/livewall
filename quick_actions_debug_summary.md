# Debug Summary: LiveWall macOS Quick Actions

## Problem
We are attempting to install two Automator Quick Actions (`.workflow`) to `~/Library/Services` that invoke a custom Rust CLI (`livewallctl`).
- **Target OS**: macOS Sonoma / Sequoia (Sequoia in this current session).
- **Issue**: 
  1. Sometimes the Quick Actions do not appear in the Finder "Quick Actions" menu.
  2. When they do appear, clicking them results in an alert: **"The Service cannot be run because it is not configured correctly."**

## Core Components
- **CLI**: [/usr/local/bin/livewallctl](file:///usr/local/bin/livewallctl) (successfully installed and working).
- **Workflows**: `SetAsLiveWallpaper.workflow` and `SetAsScreensaver.workflow`.
- **Action**: Both use a "Run Shell Script" action ([/bin/bash](file:///bin/bash)) to call the CLI.

---

## What We've Tried So Far

### 1. Permissions & Ownership
- **Action**: The workflows were initially installed using `sudo`, making them owned by `root`. We corrected this by changing ownership to the current user (`magi:staff`).
- **Result**: They appeared in the menu after the ownership change.

### 2. Info.plist NSServices Registration
- **Action**: Manually created workflows often lack an [Info.plist](file:///Users/magi/Documents/projects/livewall/quick-actions/SetAsScreensaver.workflow/Contents/Info.plist). We added one to the bundle's `Contents/` directory.
- **Variation A (Full)**: Included `NSServices` array with `NSMenuItem`, `NSSendFileTypes` (`public.movie`, etc.), and `NSMessage` (`runWorkflow`).
- **Variation B (Minimal)**: Only basic bundle info (identifier, name).
- **Result**: **Variation A** makes them visible. **Variation B** makes them disappear entirely.

### 3. Metadata Alignment ([document.wflow](file:///Users/magi/Documents/projects/livewall/quick-actions/SetAsScreensaver.workflow/Contents/document.wflow))
- **Action**: Updated the internal [document.wflow](file:///Users/magi/Documents/projects/livewall/quick-actions/SetAsScreensaver.workflow/Contents/document.wflow) XML to match modern macOS service identifiers.
- **Specific Keys Changed**:
  - `itemType`: `com.apple.automator.fileSystemObject.movie`
  - `quickActionItemType`: `com.apple.automator.fileSystemObject.movie`
  - `serviceInputTypeIdentifier`: `com.apple.automator.fileSystemObject.movie`
  - `serviceOutputTypeIdentifier`: `com.apple.automator.nothing`
- **Result**: This was intended to fix the "not configured correctly" error by strictly defining the input/output contract, but the error persists.

### 4. System Cache Refreshing
- **Action**: Running following commands to force macOS to re-index the services:
  - `/System/Library/CoreServices/pbs -update`
  - `/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f -R ~/Library/Services`
- **Result**: Inconsistent. Sometimes requires multiple runs for the menu to update.

---

## What Works
- **Manual Execution**: Running `automator -v ~/Library/Services/SetAsLiveWallpaper.workflow` from the terminal **works perfectly**. The CLI is invoked, the video is processed, and the wallpaper changes.
- **CLI**: The binary at [/usr/local/bin/livewallctl](file:///usr/local/bin/livewallctl) is confirmed working and accessible.

## What Doesn't Work
- **Finder Integration**: When triggered via the right-click menu, the "not configured correctly" error blocks execution.
- **Sequoia/Sonoma Discovery**: The OS seems extremely sensitive to the bundle structure. Even standard [Info.plist](file:///Users/magi/Documents/projects/livewall/quick-actions/SetAsScreensaver.workflow/Contents/Info.plist) keys sometimes cause the service to vanish from the "Extensions" pane in System Settings.

## Current State of Workflow (SetAsLiveWallpaper/Contents/document.wflow)
```xml
<key>workflowMetaData</key>
<dict>
    <key>serviceInputTypeIdentifier</key>
    <string>com.apple.automator.fileSystemObject.movie</string>
    <key>serviceOutputTypeIdentifier</key>
    <string>com.apple.automator.nothing</string>
    <key>workflowTypeIdentifier</key>
    <string>com.apple.Automator.servicesMenu</string>
</dict>
```

---

## Hypothesis for Claude Opus
1. **Sandboxing/Transparency**: Is there a specific `com.apple.security.*` entitlement or `NSAppleEventsUsageDescription` needed in the [Info.plist](file:///Users/magi/Documents/projects/livewall/quick-actions/SetAsScreensaver.workflow/Contents/Info.plist) for the `WorkflowServiceRunner` to execute a shell script that calls `/usr/local/bin`?
2. **Bundle Identifiers**: Is there a conflict between the `CFBundleIdentifier` in `Info.plist` and the internal metadata?
3. **Sequoia Service Logic**: Has macOS Sequoia moved away from supporting `.workflow` bundles in `~/Library/Services` in favor of some other format or location (e.g., Application Extensions)?
4. **Input Types**: Are `public.movie` and `com.apple.automator.fileSystemObject.movie` correctly registered in the system Uniform Type Identifiers for this context?
