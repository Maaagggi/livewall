use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacOSVersion {
    Sonoma,
    Sequoia,
    Tahoe,
    Unsupported(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedPaths {
    pub videos_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub thumbnails_dir: PathBuf,
    pub renderer_process: String,
    pub agent_process: String,
    pub requires_elevation: bool,
    pub macos_version: MacOSVersion,
}

pub struct PathResolver;

impl PathResolver {
    pub fn resolve() -> Result<ResolvedPaths> {
        let version = Self::get_macos_version()?;
        
        match version {
            MacOSVersion::Unsupported(v) => {
                let msg = format!(
                    "LiveWall requires macOS Sonoma (14) or later.\n\
                     Video wallpapers were introduced in macOS Sonoma.\n\
                     Your Mac is running macOS {}. Please upgrade to use LiveWall.",
                    v
                );
                Err(anyhow!(msg))
            }
            _ => {
                // Strategy 1: lsof (implementation simplified for now)
                if let Ok(paths) = Self::strategy_lsof(&version) {
                    return Ok(paths);
                }

                // Strategy 2: Priority paths
                if let Ok(paths) = Self::strategy_priority_paths(&version) {
                    return Ok(paths);
                }

                Err(anyhow!("LiveWall could not locate wallpaper storage on this system."))
            }
        }
    }

    fn get_macos_version() -> Result<MacOSVersion> {
        let output = Command::new("sw_vers")
            .arg("-productVersion")
            .output()?;
        
        let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let major = version_str.split('.').next().unwrap_or("0").parse::<u32>().unwrap_or(0);

        match major {
            14 => Ok(MacOSVersion::Sonoma),
            15 => Ok(MacOSVersion::Sequoia),
            26 => Ok(MacOSVersion::Tahoe), // Assuming Tahoe is 26 as per PRD
            _ => Ok(MacOSVersion::Unsupported(version_str)),
        }
    }

    fn strategy_lsof(_version: &MacOSVersion) -> Result<ResolvedPaths> {
        // This would involve running pgrep and lsof
        // For brevity in this initial setup, we will skip to Strategy 2 if this fails
        Err(anyhow!("lsof strategy not implemented yet"))
    }

    fn strategy_priority_paths(version: &MacOSVersion) -> Result<ResolvedPaths> {
        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME environment variable not set"))?;
        let home_path = PathBuf::from(home);

        match version {
            MacOSVersion::Tahoe => {
                let base_dir = home_path.join("Library/Application Support/com.apple.wallpaper/aerials");
                let videos_dir = base_dir.join("videos");
                let manifest_path = base_dir.join("manifest/entries.json");
                let thumbnails_dir = base_dir.join("thumbnails");

                if manifest_path.exists() {
                    Ok(ResolvedPaths {
                        videos_dir,
                        manifest_path,
                        thumbnails_dir,
                        renderer_process: "WallpaperAerialsExtension".to_string(),
                        agent_process: "WallpaperAgent".to_string(),
                        requires_elevation: false,
                        macos_version: version.clone(),
                    })
                } else {
                    Err(anyhow!("Tahoe paths not found"))
                }
            }
            MacOSVersion::Sonoma | MacOSVersion::Sequoia => {
                // System mode
                let system_base = PathBuf::from("/Library/Application Support/com.apple.idleassetsd/Customer");
                let videos_dir = system_base.join("4KSDR240FPS");
                let manifest_path = system_base.join("entries.json");
                let thumbnails_dir = system_base.join("thumbnails"); // Verify this path

                if manifest_path.exists() {
                    Ok(ResolvedPaths {
                        videos_dir,
                        manifest_path,
                        thumbnails_dir,
                        renderer_process: "WallpaperAerialExtension".to_string(),
                        agent_process: "idleassetsd".to_string(),
                        requires_elevation: true,
                        macos_version: version.clone(),
                    })
                } else {
                    // Fallback to user mode
                    let user_base = home_path.join("Library/Application Support/com.apple.idleassetsd/Customer");
                    let videos_dir = user_base.join("4KSDR240FPS");
                    let manifest_path = user_base.join("entries.json");
                    let thumbnails_dir = user_base.join("thumbnails");

                    if manifest_path.exists() {
                        Ok(ResolvedPaths {
                            videos_dir,
                            manifest_path,
                            thumbnails_dir,
                            renderer_process: "WallpaperAerialExtension".to_string(),
                            agent_process: "idleassetsd".to_string(),
                            requires_elevation: false,
                            macos_version: version.clone(),
                        })
                    } else {
                        Err(anyhow!("Sonoma/Sequoia paths not found"))
                    }
                }
            }
            _ => Err(anyhow!("Unsupported OS")),
        }
    }
}
