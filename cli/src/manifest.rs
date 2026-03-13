use std::path::Path;
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestEntry {
    #[serde(rename = "accessibilityLabel")]
    pub accessibility_label: String,
    pub categories: Vec<String>,
    pub id: String,
    #[serde(rename = "includeInShuffle")]
    pub include_in_shuffle: bool,
    #[serde(rename = "localizedNameKey")]
    pub localized_name_key: String,
    #[serde(rename = "pointsOfInterest")]
    pub points_of_interest: serde_json::Value,
    #[serde(rename = "preferredOrder")]
    pub preferred_order: i32,
    #[serde(rename = "previewImage")]
    pub preview_image: String,
    #[serde(rename = "shotID")]
    pub shot_id: String,
    #[serde(rename = "showInTopLevel")]
    pub show_in_top_level: bool,
    pub subcategories: Vec<String>,
    #[serde(rename = "url-4K-SDR-240FPS")]
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest at {:?}", path))?;
        
        // The manifest is usually an array of entries at the top level
        let entries: Vec<ManifestEntry> = serde_json::from_str(&content)
            .with_context(|| "Failed to parse entries.json")?;
        
        Ok(Manifest { entries })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        // Create backup
        let mut backup_path = path.to_path_buf();
        backup_path.set_extension("bak");
        fs::copy(path, &backup_path).context("Failed to create manifest backup")?;

        let content = serde_json::to_string_pretty(&self.entries)
            .context("Failed to serialize manifest")?;
        
        fs::write(path, content).context("Failed to write manifest")?;
        Ok(())
    }

    pub fn add_entry(&mut self, entry: ManifestEntry) {
        self.entries.push(entry);
    }

    pub fn remove_entry(&mut self, id: &str) -> bool {
        let initial_len = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < initial_len
    }

    pub fn create_entry(id: &str, name: &str, video_url: &str, thumbnail: &str) -> ManifestEntry {
        ManifestEntry {
            accessibility_label: name.to_string(),
            categories: vec![],
            id: id.to_string(),
            include_in_shuffle: true,
            localized_name_key: format!("LIVEWALL_{}", id),
            points_of_interest: serde_json::json!({}),
            preferred_order: 99999,
            preview_image: thumbnail.to_string(),
            shot_id: format!("LW_{}", &id[..8]),
            show_in_top_level: true,
            subcategories: vec![],
            url: video_url.to_string(),
            source: Some("livewall".to_string()),
        }
    }
}
