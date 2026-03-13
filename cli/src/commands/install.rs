use std::process::Command;
use std::fs;
use std::env;
use std::path::PathBuf;
use anyhow::{Result, Context};

pub fn execute_install() -> Result<()> {
    println!("Installing LiveWall...");

    // 1. Get current executable path
    let current_exe = env::current_exe().context("Failed to get current executable path")?;
    
    // 2. Install CLI binary to /usr/local/bin (requires sudo if not writable)
    // For this POC, we'll try to copy it, but in a real app, the installer would handle this with privileges
    println!("Installing CLI to /usr/local/bin/livewallctl...");
    let target_cli = PathBuf::from("/usr/local/bin/livewallctl");
    
    // Check if we can write to /usr/local/bin
    if let Err(_) = fs::copy(&current_exe, &target_cli) {
        println!("Warning: Could not copy to /usr/local/bin. You may need to run this command with sudo.");
        println!("Alternatively, you can manually copy {:?} to /usr/local/bin/livewallctl", current_exe);
    } else {
        println!("CLI installed successfully.");
    }

    // 3. Install Automator workflows
    let home = env::var("HOME").context("HOME environment variable not set")?;
    let services_dir = PathBuf::from(home).join("Library/Services");
    
    if !services_dir.exists() {
        fs::create_dir_all(&services_dir).context("Failed to create services directory")?;
    }

    // Look for quick-actions in current dir or parent dir (common in dev)
    let mut workflows_source = PathBuf::from("quick-actions");
    if !workflows_source.exists() {
        workflows_source = PathBuf::from("../quick-actions");
    }

    if workflows_source.exists() {
        println!("Found quick-actions at {:?}", workflows_source);
        for entry in fs::read_dir(&workflows_source).context("Failed to read quick-actions directory")? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("workflow") {
                let target = services_dir.join(path.file_name().unwrap());
                println!("Installing workflow: {:?}", target);
                // Clean old version if exists
                if target.exists() {
                    fs::remove_dir_all(&target).ok();
                }
                // Copy directory (Automator workflows are bundles/directories)
                copy_dir_all(&path, &target).context(format!("Failed to copy workflow {:?}", path))?;
            }
        }
        
        // 4. Update pbs
        println!("Updating Finder Quick Actions cache...");
        Command::new("/System/Library/CoreServices/pbs")
            .arg("-update")
            .status()
            .ok();
        
        println!("Workflows installed successfully.");
    } else {
        println!("Warning: quick-actions directory not found. Please ensure it exists in the current directory.");
    }

    println!("LiveWall installation complete!");
    Ok(())
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
