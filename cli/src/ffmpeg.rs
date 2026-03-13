use std::process::Command;
use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow, Context};
use which::which;

pub struct FfmpegManager;

impl FfmpegManager {
    pub fn get_ffmpeg_path() -> Result<PathBuf> {
        // 1. Try bundled ffmpeg in vendor
        let bundled = PathBuf::from("vendor/ffmpeg/ffmpeg"); // This path might need adjustment based on where the CLI is run
        if bundled.exists() {
            return Ok(bundled);
        }

        // 2. Try which
        if let Ok(path) = which("ffmpeg") {
            return Ok(path);
        }

        Err(anyhow!("ffmpeg not found. Please ensure it is bundled or installed in PATH."))
    }

    pub fn transcode_to_mov(input: &Path, output: &Path) -> Result<()> {
        let ffmpeg = Self::get_ffmpeg_path()?;
        
        // Command: ffmpeg -i input.mp4 -c:v libx264 -an -movflags faststart -y output.mov
        let status = Command::new(ffmpeg)
            .arg("-i").arg(input)
            .arg("-c:v").arg("libx264")
            .arg("-an") // Remove audio
            .arg("-movflags").arg("faststart")
            .arg("-y") // Overwrite
            .arg(output)
            .status()
            .context("Failed to execute ffmpeg for transcoding")?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("ffmpeg transcoding failed"))
        }
    }

    pub fn extract_thumbnail(input: &Path, output: &Path) -> Result<()> {
        let ffmpeg = Self::get_ffmpeg_path()?;
        
        // Command: ffmpeg -i input.mov -ss 00:00:01 -vframes 1 -y output.png
        let status = Command::new(ffmpeg)
            .arg("-i").arg(input)
            .arg("-ss").arg("00:00:01")
            .arg("-vframes").arg("1")
            .arg("-y")
            .arg(output)
            .status()
            .context("Failed to execute ffmpeg for thumbnail extraction")?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("ffmpeg thumbnail extraction failed"))
        }
    }
}
