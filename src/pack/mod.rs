use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use crate::config::MacroConfig;

/// A macro pack (profile) consisting of categorized macros.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub name: String,
    pub author: Option<String>,
    pub categories: Vec<Category>,
}

/// A named grouping of macros.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub macros: Vec<MacroConfig>,
}

impl Pack {
    /// Flatten the categories into a single list of macros.
    pub fn flatten(&self) -> Vec<MacroConfig> {
        self.categories
            .iter()
            .flat_map(|c| c.macros.clone())
            .collect()
    }

    /// Load a pack from a directory containing `pack.yaml`.
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let yaml_path = dir.join("pack.yaml");
        let file = std::fs::File::open(&yaml_path)
            .with_context(|| format!("Failed to open pack.yaml at {}", yaml_path.display()))?;
        let pack: Pack = serde_yaml_ng::from_reader(file)
            .with_context(|| format!("Failed to parse pack.yaml at {}", yaml_path.display()))?;
        Ok(pack)
    }

    /// Import a .hdpack (ZIP) file.
    pub fn import(zip_path: &Path) -> Result<Self> {
        let file = std::fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        // Find pack.yaml to get the pack name
        let mut pack_yaml_content = String::new();
        {
            let mut file = archive.by_name("pack.yaml")
                .context("ZIP archive missing pack.yaml")?;
            use std::io::Read;
            file.read_to_string(&mut pack_yaml_content)?;
        }
        
        let pack: Pack = serde_yaml_ng::from_str(&pack_yaml_content)
            .context("Failed to parse pack.yaml from ZIP")?;
        
        let dest_dir = get_profiles_dir()?.join(&pack.name);
        if dest_dir.exists() {
            std::fs::remove_dir_all(&dest_dir)?;
        }
        std::fs::create_dir_all(&dest_dir)?;

        // Extract all files with path traversal protection
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => dest_dir.join(path),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(&p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(pack)
    }

    /// Export a pack to a .hdpack (ZIP) file.
    pub fn export(&self, dest_path: &Path) -> Result<()> {
        let file = std::fs::File::create(dest_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Write pack.yaml
        zip.start_file("pack.yaml", options)?;
        let yaml = serde_yaml_ng::to_string(self)?;
        use std::io::Write;
        zip.write_all(yaml.as_bytes())?;

        // TODO: Copy sounds/ directory if it exists in the profile dir
        
        zip.finish()?;
        Ok(())
    }
}

pub fn get_profiles_dir() -> Result<PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("hd-linux-voice")?;
    let dir = xdg.get_config_home().join("profiles");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
