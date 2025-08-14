use std::path::Path;
use tokio::fs;
use serde_yaml;
use crate::pkg::config::SETTINGS;
use anyhow::Result;

/// Save the global SETTINGS back to a YAML file asynchronously
pub async fn save_config<P: AsRef<Path>>(filename: P) -> Result<()> {
    // Serialize SETTINGS to YAML string
    let data = serde_yaml::to_string(&*SETTINGS)?;

    // Write YAML string to file asynchronously (overwrites existing file)
    fs::write(filename, data).await?;

    Ok(())
}