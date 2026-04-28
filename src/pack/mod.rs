/// Profile manager — tracks the active profile and persists selection to `manager.yaml`.
pub mod manager;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{bail, Context, Result};
use crate::config::{KeyAction, MacroConfig};

/// A macro pack (profile) consisting of categorized macros.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    /// Display name of the pack, also used as the profiles directory entry name.
    pub name: String,
    /// Optional author attribution stored in `pack.yaml`.
    pub author: Option<String>,
    /// Ordered list of macro categories; macros are flattened for matching.
    pub categories: Vec<Category>,
}

/// A named grouping of macros.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Display name for this category (e.g. `"Stratagems"`).
    pub name: String,
    /// Macros belonging to this category.
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

    /// Import a .hdpack (ZIP) file, extracting into `dest_dir/<pack_name>`.
    ///
    /// The caller passes the *parent* profiles directory; this function appends
    /// the pack name itself — matching the semantics of the original `import`.
    pub fn import_to(zip_path: &Path, dest_dir: &Path) -> Result<Self> {
        tracing::info!(zip_path = %zip_path.display(), dest_dir = %dest_dir.display(), "import_to: starting");

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

        let pack_dest = dest_dir.join(&pack.name);
        if pack_dest.exists() {
            std::fs::remove_dir_all(&pack_dest)?;
        }
        std::fs::create_dir_all(&pack_dest)?;

        // Extract all files with path traversal protection
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => pack_dest.join(path),
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

        let macro_count = pack.flatten().len();
        tracing::info!(zip_path = %zip_path.display(), dest_dir = %pack_dest.display(), macro_count, "import_to: done");

        Ok(pack)
    }

    /// Import a .hdpack (ZIP) file into the default XDG profiles directory.
    pub fn import(zip_path: &Path) -> Result<Self> {
        let profiles_dir = get_profiles_dir()?;
        Self::import_to(zip_path, &profiles_dir)
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

/// Return the XDG profiles directory (`$XDG_CONFIG_HOME/vibe-attack/profiles`), creating it if absent.
pub fn get_profiles_dir() -> Result<PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("vibe-attack");
    let config_home = xdg.get_config_home()
        .context("Failed to determine config directory")?;
    let dir = config_home.join("profiles");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Partial-update descriptor for [`PackEditor::edit_macro`].
///
/// `None` for a field means "leave unchanged". `Some(None)` clears an optional field.
/// `Some(Some(v))` sets the field to `v`. The `keys` field replaces the entire vec when `Some`.
#[derive(Debug, Clone, Default)]
pub struct MacroUpdates {
    pub phrase: Option<Option<String>>,
    pub if_flag: Option<Option<String>>,
    pub set_flag: Option<Option<String>>,
    pub sound: Option<Option<PathBuf>>,
    pub keys: Option<Vec<KeyAction>>,
}

/// Mutable wrapper around a [`Pack`] that enforces invariants on every CRUD operation.
///
/// Macro name uniqueness is enforced within a category, not globally.
pub struct PackEditor {
    pack: Pack,
}

impl PackEditor {
    pub fn new(pack: Pack) -> Self {
        Self { pack }
    }

    pub fn pack(&self) -> &Pack {
        &self.pack
    }

    pub fn into_pack(self) -> Pack {
        self.pack
    }

    /// Append `macro_config` to the end of the named category's macro list.
    ///
    /// Errors if `category` does not exist or if a macro with the same name is already present.
    pub fn add_macro(&mut self, category: &str, macro_config: MacroConfig) -> Result<()> {
        let cat = self
            .pack
            .categories
            .iter_mut()
            .find(|c| c.name == category)
            .ok_or_else(|| anyhow::anyhow!("category '{}' not found", category))?;

        if cat.macros.iter().any(|m| m.name == macro_config.name) {
            bail!(
                "macro '{}' already exists in category '{}'",
                macro_config.name,
                category
            );
        }

        cat.macros.push(macro_config);
        Ok(())
    }

    /// Apply `updates` to the named macro in `category`, leaving all other fields unchanged.
    ///
    /// Errors if `category` or `macro_name` is not found.
    pub fn edit_macro(
        &mut self,
        category: &str,
        macro_name: &str,
        updates: MacroUpdates,
    ) -> Result<()> {
        let cat = self
            .pack
            .categories
            .iter_mut()
            .find(|c| c.name == category)
            .ok_or_else(|| anyhow::anyhow!("category '{}' not found", category))?;

        let m = cat
            .macros
            .iter_mut()
            .find(|m| m.name == macro_name)
            .ok_or_else(|| {
                anyhow::anyhow!("macro '{}' not found in category '{}'", macro_name, category)
            })?;

        if let Some(phrase) = updates.phrase {
            m.phrase = phrase;
        }
        if let Some(if_flag) = updates.if_flag {
            m.if_flag = if_flag;
        }
        if let Some(set_flag) = updates.set_flag {
            m.set_flag = set_flag;
        }
        if let Some(sound) = updates.sound {
            m.sound = sound;
        }
        if let Some(keys) = updates.keys {
            m.keys = keys;
        }

        Ok(())
    }

    /// Remove the named macro from `category`.
    ///
    /// Errors if `category` or `macro_name` is not found.
    pub fn remove_macro(&mut self, category: &str, macro_name: &str) -> Result<()> {
        let cat = self
            .pack
            .categories
            .iter_mut()
            .find(|c| c.name == category)
            .ok_or_else(|| anyhow::anyhow!("category '{}' not found", category))?;

        let idx = cat
            .macros
            .iter()
            .position(|m| m.name == macro_name)
            .ok_or_else(|| {
                anyhow::anyhow!("macro '{}' not found in category '{}'", macro_name, category)
            })?;

        cat.macros.remove(idx);
        Ok(())
    }

    /// Move `macro_name` from `from_category` to the end of `to_category`.
    ///
    /// Atomic: validates all preconditions before any mutation.
    /// Errors if either category is missing, the macro is missing from source,
    /// or a macro with the same name already exists in the destination.
    pub fn move_macro(
        &mut self,
        from_category: &str,
        to_category: &str,
        macro_name: &str,
    ) -> Result<()> {
        // Validate preconditions before any mutation.
        let from_idx = self
            .pack
            .categories
            .iter()
            .position(|c| c.name == from_category)
            .ok_or_else(|| anyhow::anyhow!("source category '{}' not found", from_category))?;

        let to_idx = self
            .pack
            .categories
            .iter()
            .position(|c| c.name == to_category)
            .ok_or_else(|| anyhow::anyhow!("destination category '{}' not found", to_category))?;

        let macro_idx = self.pack.categories[from_idx]
            .macros
            .iter()
            .position(|m| m.name == macro_name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "macro '{}' not found in category '{}'",
                    macro_name,
                    from_category
                )
            })?;

        if self.pack.categories[to_idx]
            .macros
            .iter()
            .any(|m| m.name == macro_name)
        {
            bail!(
                "macro '{}' already exists in destination category '{}'",
                macro_name,
                to_category
            );
        }

        // All checks passed — mutate atomically.
        let moved = self.pack.categories[from_idx].macros.remove(macro_idx);
        self.pack.categories[to_idx].macros.push(moved);
        Ok(())
    }

    /// Rename `old_name` category to `new_name` in place (preserves index and macros).
    ///
    /// Errors if `old_name` is not found, if `new_name == old_name`, or if `new_name` is already in use.
    pub fn rename_category(&mut self, old_name: &str, new_name: &str) -> Result<()> {
        if old_name == new_name {
            bail!("new category name '{}' is identical to the old name", new_name);
        }

        if self.pack.categories.iter().any(|c| c.name == new_name) {
            bail!("category '{}' already exists", new_name);
        }

        let cat = self
            .pack
            .categories
            .iter_mut()
            .find(|c| c.name == old_name)
            .ok_or_else(|| anyhow::anyhow!("category '{}' not found", old_name))?;

        cat.name = new_name.to_string();
        Ok(())
    }

    /// Append a new empty category with `name` to the end of `pack.categories`.
    ///
    /// Errors if `name` is already in use.
    pub fn add_category(&mut self, name: &str) -> Result<()> {
        if self.pack.categories.iter().any(|c| c.name == name) {
            bail!("category '{}' already exists", name);
        }

        self.pack.categories.push(Category {
            name: name.to_string(),
            macros: vec![],
        });
        Ok(())
    }

    /// Remove the named category.
    ///
    /// Errors if `name` is not found or if the category still contains macros
    /// (caller must explicitly empty it first via `remove_macro`).
    pub fn remove_category(&mut self, name: &str) -> Result<()> {
        let idx = self
            .pack
            .categories
            .iter()
            .position(|c| c.name == name)
            .ok_or_else(|| anyhow::anyhow!("category '{}' not found", name))?;

        if !self.pack.categories[idx].macros.is_empty() {
            bail!(
                "category '{}' still has {} macro(s); empty it first",
                name,
                self.pack.categories[idx].macros.len()
            );
        }

        self.pack.categories.remove(idx);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // -----------------------------------------------------------------------
    // PackEditor fixture
    // -----------------------------------------------------------------------

    fn editor_key(name: &str) -> KeyAction {
        KeyAction { key: name.to_string(), dwell_ms: None, gap_ms: None }
    }

    fn editor_macro(name: &str, phrase: Option<&str>) -> MacroConfig {
        MacroConfig {
            name: name.to_string(),
            phrase: phrase.map(str::to_string),
            if_flag: None,
            set_flag: None,
            sound: None,
            keys: vec![editor_key("KEY_W")],
        }
    }

    /// Two categories, two macros each.
    fn fixture_pack() -> Pack {
        Pack {
            name: "Fixture".to_string(),
            author: None,
            categories: vec![
                Category {
                    name: "Stratagems".to_string(),
                    macros: vec![
                        editor_macro("Reinforce", Some("reinforce")),
                        editor_macro("Resupply", Some("resupply")),
                    ],
                },
                Category {
                    name: "Ship Modules".to_string(),
                    macros: vec![
                        editor_macro("Shield Gen", Some("shield generator")),
                        editor_macro("Extra Padding", Some("extra padding")),
                    ],
                },
            ],
        }
    }

    // -----------------------------------------------------------------------
    // add_macro tests
    // -----------------------------------------------------------------------

    #[test]
    fn editor_add_macro_success() {
        let mut ed = PackEditor::new(fixture_pack());
        let new_macro = editor_macro("Eagle Airstrike", Some("eagle airstrike"));
        ed.add_macro("Stratagems", new_macro).expect("add_macro must succeed");

        let cat = ed.pack().categories.iter().find(|c| c.name == "Stratagems").unwrap();
        assert_eq!(cat.macros.len(), 3);
        assert_eq!(cat.macros.last().unwrap().name, "Eagle Airstrike");
    }

    #[test]
    fn editor_add_macro_unknown_category_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.add_macro("Nonexistent", editor_macro("Foo", None)).unwrap_err();
        assert!(err.to_string().contains("Nonexistent"), "error must name the missing category");
    }

    #[test]
    fn editor_add_macro_duplicate_name_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.add_macro("Stratagems", editor_macro("Reinforce", Some("reinforce"))).unwrap_err();
        assert!(err.to_string().contains("Reinforce"), "error must name the duplicate macro");
        assert!(err.to_string().contains("Stratagems"), "error must name the category");
    }

    // -----------------------------------------------------------------------
    // edit_macro tests
    // -----------------------------------------------------------------------

    #[test]
    fn editor_edit_macro_replaces_phrase_and_keys() {
        let mut ed = PackEditor::new(fixture_pack());
        let new_key = editor_key("KEY_S");
        ed.edit_macro(
            "Stratagems",
            "Reinforce",
            MacroUpdates {
                phrase: Some(Some("new phrase".to_string())),
                keys: Some(vec![new_key]),
                ..Default::default()
            },
        )
        .expect("edit_macro must succeed");

        let cat = ed.pack().categories.iter().find(|c| c.name == "Stratagems").unwrap();
        let m = cat.macros.iter().find(|m| m.name == "Reinforce").unwrap();
        assert_eq!(m.phrase, Some("new phrase".to_string()));
        assert_eq!(m.keys.len(), 1);
        assert_eq!(m.keys[0].key, "KEY_S");
        // untouched fields preserved
        assert_eq!(m.if_flag, None);
        assert_eq!(m.set_flag, None);
        assert_eq!(m.sound, None);
    }

    #[test]
    fn editor_edit_macro_can_clear_optional_field() {
        let mut ed = PackEditor::new(fixture_pack());
        // phrase starts as Some("reinforce") — clear it to None
        ed.edit_macro(
            "Stratagems",
            "Reinforce",
            MacroUpdates {
                phrase: Some(None),
                ..Default::default()
            },
        )
        .expect("edit_macro must succeed");

        let cat = ed.pack().categories.iter().find(|c| c.name == "Stratagems").unwrap();
        let m = cat.macros.iter().find(|m| m.name == "Reinforce").unwrap();
        assert_eq!(m.phrase, None, "phrase must be cleared to None");
    }

    #[test]
    fn editor_edit_macro_unknown_category_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed
            .edit_macro("Ghost Category", "Reinforce", MacroUpdates::default())
            .unwrap_err();
        assert!(err.to_string().contains("Ghost Category"));
    }

    #[test]
    fn editor_edit_macro_unknown_macro_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed
            .edit_macro("Stratagems", "Ghost Macro", MacroUpdates::default())
            .unwrap_err();
        assert!(err.to_string().contains("Ghost Macro"));
        assert!(err.to_string().contains("Stratagems"));
    }

    // -----------------------------------------------------------------------
    // remove_macro tests
    // -----------------------------------------------------------------------

    #[test]
    fn editor_remove_macro_success() {
        let mut ed = PackEditor::new(fixture_pack());
        ed.remove_macro("Stratagems", "Reinforce").expect("remove_macro must succeed");

        let cat = ed.pack().categories.iter().find(|c| c.name == "Stratagems").unwrap();
        assert_eq!(cat.macros.len(), 1, "one macro must remain");
        assert!(cat.macros.iter().all(|m| m.name != "Reinforce"), "Reinforce must be absent");
    }

    #[test]
    fn editor_remove_macro_unknown_category_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.remove_macro("Phantom", "Reinforce").unwrap_err();
        assert!(err.to_string().contains("Phantom"));
    }

    #[test]
    fn editor_remove_macro_unknown_macro_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.remove_macro("Stratagems", "Phantom Macro").unwrap_err();
        assert!(err.to_string().contains("Phantom Macro"));
        assert!(err.to_string().contains("Stratagems"));
    }

    // -----------------------------------------------------------------------
    // move_macro tests
    // -----------------------------------------------------------------------

    #[test]
    fn editor_move_macro_success() {
        let mut ed = PackEditor::new(fixture_pack());
        ed.move_macro("Stratagems", "Ship Modules", "Reinforce").expect("move_macro must succeed");

        let src = ed.pack().categories.iter().find(|c| c.name == "Stratagems").unwrap();
        assert!(src.macros.iter().all(|m| m.name != "Reinforce"), "macro must be absent from source");

        let dst = ed.pack().categories.iter().find(|c| c.name == "Ship Modules").unwrap();
        assert_eq!(dst.macros.last().unwrap().name, "Reinforce", "macro must appear at end of dest");
    }

    #[test]
    fn editor_move_macro_unknown_source_category_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.move_macro("Nonexistent", "Ship Modules", "Reinforce").unwrap_err();
        assert!(err.to_string().contains("Nonexistent"), "error must name the missing source category");
    }

    #[test]
    fn editor_move_macro_unknown_dest_category_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.move_macro("Stratagems", "Nonexistent", "Reinforce").unwrap_err();
        assert!(err.to_string().contains("Nonexistent"), "error must name the missing dest category");
    }

    #[test]
    fn editor_move_macro_unknown_macro_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.move_macro("Stratagems", "Ship Modules", "Ghost Macro").unwrap_err();
        assert!(err.to_string().contains("Ghost Macro"), "error must name the missing macro");
    }

    #[test]
    fn editor_move_macro_duplicate_in_dest_errors_and_leaves_source_unchanged() {
        let mut ed = PackEditor::new(fixture_pack());
        // Pre-populate dest with the same name to trigger the duplicate check.
        ed.add_macro("Ship Modules", editor_macro("Reinforce", None)).expect("pre-populate must succeed");

        let err = ed.move_macro("Stratagems", "Ship Modules", "Reinforce").unwrap_err();
        assert!(err.to_string().contains("Reinforce"), "error must name the duplicate macro");

        // Atomicity check: source must still contain the macro.
        let src = ed.pack().categories.iter().find(|c| c.name == "Stratagems").unwrap();
        assert!(
            src.macros.iter().any(|m| m.name == "Reinforce"),
            "source category must still contain Reinforce after failed move"
        );
    }

    // -----------------------------------------------------------------------
    // rename_category tests
    // -----------------------------------------------------------------------

    #[test]
    fn editor_rename_category_success() {
        let mut ed = PackEditor::new(fixture_pack());
        let original_idx = ed.pack().categories.iter().position(|c| c.name == "Stratagems").unwrap();

        ed.rename_category("Stratagems", "Tactics").expect("rename_category must succeed");

        assert!(
            ed.pack().categories.iter().all(|c| c.name != "Stratagems"),
            "old name must be absent"
        );
        let renamed = ed.pack().categories.iter().find(|c| c.name == "Tactics").unwrap();
        assert_eq!(
            ed.pack().categories.iter().position(|c| c.name == "Tactics").unwrap(),
            original_idx,
            "renamed category must stay at the same index"
        );
        assert_eq!(renamed.macros.len(), 2, "macros must be preserved after rename");
    }

    #[test]
    fn editor_rename_category_unknown_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.rename_category("Ghost Category", "New Name").unwrap_err();
        assert!(err.to_string().contains("Ghost Category"));
    }

    #[test]
    fn editor_rename_category_duplicate_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.rename_category("Stratagems", "Ship Modules").unwrap_err();
        assert!(err.to_string().contains("Ship Modules"), "error must name the duplicate category");
    }

    // -----------------------------------------------------------------------
    // add_category tests
    // -----------------------------------------------------------------------

    #[test]
    fn editor_add_category_success() {
        let mut ed = PackEditor::new(fixture_pack());
        let before_len = ed.pack().categories.len();
        ed.add_category("Boosters").expect("add_category must succeed");

        assert_eq!(ed.pack().categories.len(), before_len + 1, "categories must grow by one");
        let added = ed.pack().categories.last().unwrap();
        assert_eq!(added.name, "Boosters", "new category must be appended at end");
        assert!(added.macros.is_empty(), "new category must be empty");
    }

    #[test]
    fn editor_add_category_duplicate_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.add_category("Stratagems").unwrap_err();
        assert!(err.to_string().contains("Stratagems"));
    }

    // -----------------------------------------------------------------------
    // remove_category tests
    // -----------------------------------------------------------------------

    #[test]
    fn editor_remove_category_success() {
        let mut ed = PackEditor::new(fixture_pack());
        // Empty the category first.
        ed.remove_macro("Stratagems", "Reinforce").unwrap();
        ed.remove_macro("Stratagems", "Resupply").unwrap();
        let before_len = ed.pack().categories.len();

        ed.remove_category("Stratagems").expect("remove_category must succeed on empty category");
        assert_eq!(ed.pack().categories.len(), before_len - 1);
        assert!(ed.pack().categories.iter().all(|c| c.name != "Stratagems"));
    }

    #[test]
    fn editor_remove_category_non_empty_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.remove_category("Stratagems").unwrap_err();
        assert!(
            err.to_string().contains("Stratagems"),
            "error must name the non-empty category"
        );
    }

    #[test]
    fn editor_remove_category_unknown_errors() {
        let mut ed = PackEditor::new(fixture_pack());
        let err = ed.remove_category("Ghost Category").unwrap_err();
        assert!(err.to_string().contains("Ghost Category"));
    }

    // -----------------------------------------------------------------------
    // Existing Pack tests below
    // -----------------------------------------------------------------------

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

    #[test]
    fn test_import_to_extracts_into_dest_dir() -> Result<()> {
        // Build a small pack with one macro.
        let source_dir = tempdir()?;
        let pack = Pack {
            name: "ImportToTest".to_string(),
            author: Some("tester".to_string()),
            categories: vec![Category {
                name: "Actions".to_string(),
                macros: vec![editor_macro("Reinforce", Some("reinforce"))],
            }],
        };
        pack.save_to_dir(source_dir.path())?;

        // Export to a zip in a temp dir.
        let zip_dir = tempdir()?;
        let zip_path = zip_dir.path().join("importtotest.hdpack");
        pack.export(source_dir.path(), &zip_path)?;

        // import_to into a separate temp dir — no XDG_CONFIG_HOME mutation.
        let dest_root = tempdir()?;
        let imported = Pack::import_to(&zip_path, dest_root.path())?;

        assert_eq!(imported.name, "ImportToTest");
        assert_eq!(imported.categories.len(), 1);
        assert_eq!(imported.categories[0].macros[0].name, "Reinforce");

        // Verify files landed under dest_root/<pack_name>.
        let extracted = dest_root.path().join("ImportToTest");
        assert!(extracted.join("pack.yaml").exists(), "pack.yaml must be extracted");

        Ok(())
    }
}

