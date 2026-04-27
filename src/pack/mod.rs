pub mod manager;

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

    /// Save the pack to a directory as `pack.yaml`.
    pub fn save_to_dir(&self, dir: &Path) -> Result<()> {
        let yaml_path = dir.join("pack.yaml");
        let file = std::fs::File::create(&yaml_path)
            .with_context(|| format!("Failed to create pack.yaml at {}", yaml_path.display()))?;
        serde_yaml_ng::to_writer(file, self)
            .with_context(|| format!("Failed to serialize pack.yaml to {}", yaml_path.display()))?;
        Ok(())
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
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(pack)
    }

    /// Export a pack to a .hdpack (ZIP) file.
    pub fn export(&self, source_dir: &Path, dest_path: &Path) -> Result<()> {
        let file = std::fs::File::create(dest_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Write pack.yaml
        zip.start_file("pack.yaml", options)?;
        let yaml = serde_yaml_ng::to_string(self)?;
        use std::io::Write;
        zip.write_all(yaml.as_bytes())?;

        // Copy sounds/ directory if it exists in the profile dir
        let sounds_dir = source_dir.join("sounds");
        if sounds_dir.exists() && sounds_dir.is_dir() {
            Self::add_dir_to_zip(&mut zip, &sounds_dir, Path::new("sounds"), options)?;
        }
        
        zip.finish()?;
        Ok(())
    }

    fn add_dir_to_zip<W: std::io::Write + std::io::Seek>(
        zip: &mut zip::ZipWriter<W>,
        real_path: &Path,
        zip_path: &Path,
        options: zip::write::FileOptions,
    ) -> Result<()> {
        for entry in std::fs::read_dir(real_path)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let new_zip_path = zip_path.join(name);

            if path.is_dir() {
                // ZIP directories must end with /
                let mut dir_name = new_zip_path.to_string_lossy().into_owned();
                if !dir_name.ends_with('/') {
                    dir_name.push('/');
                }
                zip.add_directory(dir_name, options)?;
                Self::add_dir_to_zip(zip, &path, &new_zip_path, options)?;
            } else {
                zip.start_file(new_zip_path.to_string_lossy(), options)?;
                let mut f = std::fs::File::open(path)?;
                std::io::copy(&mut f, zip)?;
            }
        }
        Ok(())
    }
}

pub fn get_profiles_dir() -> Result<PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("vibe-attack");
    let config_home = xdg.get_config_home()
        .context("Failed to determine config directory")?;
    let dir = config_home.join("profiles");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pack_save_load() -> Result<()> {
        let dir = tempdir()?;
        let pack = Pack {
            name: "Test Pack".to_string(),
            author: Some("Author".to_string()),
            categories: vec![Category {
                name: "Cat1".to_string(),
                macros: vec![],
            }],
        };

        pack.save_to_dir(dir.path())?;
        let loaded = Pack::load_from_dir(dir.path())?;

        assert_eq!(loaded.name, "Test Pack");
        assert_eq!(loaded.author, Some("Author".to_string()));
        assert_eq!(loaded.categories.len(), 1);
        Ok(())
    }

    #[test]
    fn test_pack_export_import_with_sounds() -> Result<()> {
        let source_dir = tempdir()?;
        let sounds_dir = source_dir.path().join("sounds");
        std::fs::create_dir_all(&sounds_dir)?;
        
        let sound_file = sounds_dir.join("test.wav");
        std::fs::write(&sound_file, b"test audio content")?;

        let pack = Pack {
            name: "ExportPack".to_string(),
            author: None,
            categories: vec![],
        };
        pack.save_to_dir(source_dir.path())?;

        let zip_path = source_dir.path().join("test.hdpack");
        pack.export(source_dir.path(), &zip_path)?;

        // Import it back
        // Note: import() uses get_profiles_dir() which points to XDG_CONFIG_HOME
        // For testing, we might want to mock get_profiles_dir() or just check the ZIP content directly first.
        // But let's try to run it and see if it works with the environment variables.
        
        let imported_pack = Pack::import(&zip_path)?;
        assert_eq!(imported_pack.name, "ExportPack");

        let profile_dir = get_profiles_dir()?.join("ExportPack");
        assert!(profile_dir.join("pack.yaml").exists());
        assert!(profile_dir.join("sounds/test.wav").exists());
        assert_eq!(std::fs::read_to_string(profile_dir.join("sounds/test.wav"))?, "test audio content");

        // Clean up
        std::fs::remove_dir_all(profile_dir)?;

        Ok(())
    }
}

