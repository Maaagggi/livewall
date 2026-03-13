use std::process::Command;
use std::thread;
use std::time::Duration;
use anyhow::{Result, anyhow};

pub struct DaemonManager;

impl DaemonManager {
    pub fn restart_daemons(renderer: &str, agent: &str) -> Result<()> {
        println!("Restarting daemons: {} and {}", renderer, agent);
        
        // Kill renderer
        Self::kill_process(renderer)?;
        thread::sleep(Duration::from_secs(1));
        
        // Kill agent
        Self::kill_process(agent)?;
        thread::sleep(Duration::from_secs(1));
        
        // Wait and verify (simple pgrep check)
        if Self::is_running(renderer) && Self::is_running(agent) {
            println!("Daemons restarted successfully.");
            Ok(())
        } else {
            // They might still be starting up, wait a bit longer
            thread::sleep(Duration::from_secs(2));
            if Self::is_running(renderer) && Self::is_running(agent) {
                println!("Daemons restarted successfully (delayed).");
                Ok(())
            } else {
                // Not a fatal error since they are launchd managed and will eventually start
                println!("Warning: Daemons did not reappear immediately, but launchd should restart them.");
                Ok(())
            }
        }
    }

    fn kill_process(name: &str) -> Result<()> {
        let output = Command::new("killall")
            .arg("-TERM")
            .arg(name)
            .output()?;
        
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("No matching processes belonging to you were found") {
                // Process not running, that's fine
                Ok(())
            } else {
                Err(anyhow!("Failed to kill {}: {}", name, stderr))
            }
        }
    }

    fn is_running(name: &str) -> bool {
        let output = Command::new("pgrep")
            .arg(name)
            .output();
        
        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
}
