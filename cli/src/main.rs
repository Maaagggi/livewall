use clap::{Parser, Subcommand};
use anyhow::Result;

mod path_resolver;
mod manifest;
mod daemon;
mod ffmpeg;
mod commands;

// use path_resolver::PathResolver;

#[derive(Parser)]
#[command(name = "livewallctl")]
#[command(about = "Set any video as a live wallpaper or screensaver on macOS.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set a video as live wallpaper
    SetWallpaper {
        #[arg(value_name = "FILE")]
        file: std::path::PathBuf,
    },
    /// Set a video as screensaver
    SetScreensaver {
        #[arg(value_name = "FILE")]
        file: std::path::PathBuf,
    },
    /// List all custom live wallpapers
    List,
    /// Remove a custom live wallpaper
    Remove {
        #[arg(value_name = "ID")]
        id: String,
    },
    /// Restore a backed up video (for Approach B)
    Restore {
        #[arg(value_name = "ID")]
        id: String,
    },
    /// Run initial installation
    Install,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SetWallpaper { file } => {
            commands::set::execute_set(file, true)?;
        }
        Commands::SetScreensaver { file } => {
            commands::set::execute_set(file, false)?;
        }
        Commands::List => {
            commands::list::execute_list()?;
        }
        Commands::Remove { id } => {
            commands::remove::execute_remove(id)?;
        }
        Commands::Restore { id } => {
            println!("Restoring wallpaper: {}", id);
            // Execute restore logic
        }
        Commands::Install => {
            commands::install::execute_install()?;
        }
    }

    Ok(())
}
