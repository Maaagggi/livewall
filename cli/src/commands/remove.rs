use crate::path_resolver::PathResolver;
use crate::manifest::Manifest;
use crate::daemon::DaemonManager;
use std::fs;
use anyhow::Result;

pub fn execute_remove(id: String) -> Result<()> {
    let resolved = PathResolver::resolve()?;
    let mut manifest = Manifest::load(&resolved.manifest_path)?;

    // Find the entry before removing it to get filenames
    let entry = manifest.entries.iter()
        .find(|e| e.id == id && e.source.as_deref() == Some("livewall"))
        .cloned();

    if let Some(entry) = entry {
        println!("Removing wallpaper: {} ({})", entry.accessibility_label, id);
        
        // Remove from manifest
        manifest.remove_entry(&id);
        manifest.save(&resolved.manifest_path)?;

        // Delete video file
        let video_path = resolved.videos_dir.join(format!("{}.mov", id));
        if video_path.exists() {
            fs::remove_file(video_path).ok();
        }

        // Delete thumbnail
        let thumbnail_path = resolved.thumbnails_dir.join(format!("{}.png", id));
        if thumbnail_path.exists() {
            fs::remove_file(thumbnail_path).ok();
        }

        // Restart daemons
        DaemonManager::restart_daemons(&resolved.renderer_process, &resolved.agent_process)?;
        println!("Removed successfully.");
    } else {
        println!("Wallpaper with ID {} not found or is not a LiveWall entry.", id);
    }

    Ok(())
}
