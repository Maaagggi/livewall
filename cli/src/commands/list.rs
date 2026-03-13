use crate::path_resolver::PathResolver;
use crate::manifest::Manifest;
use anyhow::Result;

pub fn execute_list() -> Result<()> {
    let resolved = PathResolver::resolve()?;
    let manifest = Manifest::load(&resolved.manifest_path)?;

    let custom_entries: Vec<_> = manifest.entries.iter()
        .filter(|e| e.source.as_deref() == Some("livewall"))
        .collect();

    if custom_entries.is_empty() {
        println!("No custom live wallpapers found.");
    } else {
        println!("{:<40} {:<30} {:<20}", "ID", "Name", "File");
        println!("{}", "-".repeat(90));
        for entry in custom_entries {
            println!("{:<40} {:<30} {:<20}", entry.id, entry.accessibility_label, entry.preview_image);
        }
    }

    Ok(())
}
