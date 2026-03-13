use std::path::PathBuf;
use std::fs;
use anyhow::{Result, anyhow, Context};
use uuid::Uuid;

use crate::path_resolver::PathResolver;
use crate::manifest::Manifest;
use crate::ffmpeg::FfmpegManager;
use crate::daemon::DaemonManager;

pub fn execute_set(file: PathBuf, is_wallpaper: bool) -> Result<()> {
    // 1. Resolve paths
    let resolved = PathResolver::resolve()?;
    println!("Resolved paths: {:?}", resolved);

    // 2. Validate input
    if !file.exists() {
        return Err(anyhow!("Input file does not exist: {:?}", file));
    }
    
    let extension = file.extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("File has no extension"))?
        .to_lowercase();
    
    if extension != "mp4" && extension != "mov" {
        return Err(anyhow!("Unsupported file type: {}. Use .mp4 or .mov", extension));
    }

    // 3. Generate UUID
    let id = Uuid::new_v4().to_string();
    let file_name = file.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("video");

    // 4. Transcode if needed, or just copy
    let target_video_path = resolved.videos_dir.join(format!("{}.mov", id));
    
    // Ensure videos_dir exists (especially on Tahoe user path)
    if !resolved.videos_dir.exists() {
        fs::create_dir_all(&resolved.videos_dir).context("Failed to create videos directory")?;
    }

    if extension == "mp4" {
        println!("Transcoding MP4 to MOV...");
        FfmpegManager::transcode_to_mov(&file, &target_video_path)?;
    } else {
        println!("Copying MOV file...");
        fs::copy(&file, &target_video_path).context("Failed to copy video file")?;
    }

    // 5. Extract thumbnail
    if !resolved.thumbnails_dir.exists() {
        fs::create_dir_all(&resolved.thumbnails_dir).context("Failed to create thumbnails directory")?;
    }
    let thumbnail_path = resolved.thumbnails_dir.join(format!("{}.png", id));
    println!("Extracting thumbnail...");
    FfmpegManager::extract_thumbnail(&target_video_path, &thumbnail_path)?;

    // 6. Patch manifest
    println!("Patching manifest...");
    let mut manifest = Manifest::load(&resolved.manifest_path)?;
    
    // Approach A: file:// URL (Test first)
    let video_url = format!("file://{}", target_video_path.to_string_lossy());
    let entry = Manifest::create_entry(&id, file_name, &video_url, &format!("{}.png", id));
    
    manifest.add_entry(entry);
    manifest.save(&resolved.manifest_path)?;

    // 7. Restart daemons
    println!("Restarting daemons...");
    DaemonManager::restart_daemons(&resolved.renderer_process, &resolved.agent_process)?;

    // 8. Success!
    if is_wallpaper {
        println!("Live wallpaper set successfully!");
        // Open system settings
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.desktopscreeneffect")
            .spawn()
            .ok();
    } else {
        println!("Screensaver set successfully!");
        // Open system settings (screensaver tab)
        // Command::new("open")
        //     .arg("x-apple.systempreferences:com.apple.preference.desktopscreeneffect?ScreenSaver")
        //     .spawn()
        //     .ok();
    }

    Ok(())
}

use std::process::Command;
