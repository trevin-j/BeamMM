use crate::{game::ModCfg, Error::*, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    ffi::OsStr,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

/// A preset of mods suitable for enabling/disabling groups of mods.
///
/// Presets are stored as JSON files in the BeamMM/presets directory.
///
/// # Examples
/// ```rust
/// use beam_mm::Preset;
/// # use tempfile::tempdir;
///
/// # let temp_dir = tempdir().unwrap();
/// # let presets_dir = temp_dir.path();
///
/// let mods: Vec<String> = vec!["mod1".into(), "mod2".into()];
///
/// // Create a preset
/// let mut new_preset = Preset::new("preset_name".into(), mods.clone());
/// new_preset.save_to_path(&presets_dir).unwrap();
///
/// // Load a preset
/// let loaded_preset = Preset::load_from_path("preset_name", &presets_dir).unwrap();
/// assert_eq!(loaded_preset.get_mods(), &mods);
/// ```
///
/// See additional preset examples in each function's documentation.
#[derive(Serialize, Deserialize)]
pub struct Preset {
    /// The name of the preset.
    name: String,
    /// The mods in the preset.
    mods: Vec<String>,
    /// Whether the preset is enabled.
    enabled: bool,
}

impl Preset {
    /// Get an iterator over currently saved presets.
    ///
    /// # Arguments
    ///
    /// `presets_dir`: Where preset config files are stored.
    ///
    /// # Errors
    ///
    /// Possible IO errors if the path doesn't exist, there is a permission issue,
    /// or if the path is not a directory.
    pub fn list(presets_dir: &Path) -> Result<impl Iterator<Item = String>> {
        Ok(fs::read_dir(presets_dir)?
            .filter_map(|f| f.ok().map(|f| f.path())) // Get rid of errors and map to path type
            .filter(|f| f.is_file() && f.extension().unwrap_or(OsStr::new("")) == "json") // Filter out dirs and non-json files
            // Map to remove the json extension so we just have the preset name and convert to String
            // if the os string into_string fails, it gets converted to None which gets filtered out
            .filter_map(|f| f.with_extension("").into_os_string().into_string().ok()))
    }

    /// Create a new preset.
    ///
    /// # Arguments
    ///
    /// `name`: The name of the preset.
    /// `mods`: The mods to include in the preset.
    pub fn new(name: String, mods: Vec<String>) -> Self {
        Preset {
            name,
            mods,
            enabled: false,
        }
    }

    /// Serialize and save the preset to a writer.
    ///
    /// # Arguments
    ///
    /// `writer`: The writer to save the preset to.
    ///
    /// # Errors
    ///
    /// Possible IO errors if there is an issue writing to the writer.
    pub fn save<W: Write>(&self, mut writer: W) -> Result<()> {
        serde_json::to_writer_pretty(&mut writer, self)?;
        writer.flush()?;

        Ok(())
    }

    /// Serialize and save the preset to a file.
    ///
    /// # Arguments
    ///
    /// `presets_dir`: The directory where the preset will be saved.
    ///
    /// # Errors
    ///
    /// Possible IO errors if there is an issue creating the file or writing to it.
    pub fn save_to_path(&self, presets_dir: &Path) -> Result<()> {
        let file = File::create(presets_dir.join(&self.name))?;
        let writer = BufWriter::new(file);
        self.save(writer)
    }

    /// Deserialize and load a preset from a reader.
    ///
    /// # Arguments
    ///
    /// `reader`: The reader to load the preset from.
    ///
    /// # Errors
    ///
    /// Possible serde_json errors if there is an issue reading or deserializing the preset.
    pub fn load<R: BufRead>(reader: R) -> Result<Self> {
        Ok(serde_json::from_reader(reader)?)
    }

    /// Deserialize and load a preset from a file.
    ///
    /// # Arguments
    ///
    /// `name`: The name of the preset to load.
    /// `presets_dir`: The directory where the preset is stored.
    ///
    /// # Errors
    ///
    /// Possible IO errors if there is an issue reading the file or serde_json errors if there is
    /// an issue deserializing the preset.
    pub fn load_from_path(name: &str, presets_dir: &Path) -> Result<Self> {
        let preset_path = presets_dir.join(name);
        if preset_path.try_exists()? {
            let file = File::open(preset_path)?;
            let reader = BufReader::new(file);
            Self::load(reader)
        } else {
            Err(MissingPreset {
                dir: presets_dir.into(),
                preset: name.into(),
            })
        }
    }

    /// Delete a preset.
    ///
    /// # Arguments
    ///
    /// `name`: The name of the preset to delete.
    /// `presets_dir`: The directory where the preset is stored.
    ///
    /// # Errors
    ///
    /// Possible IO errors if there is an issue deleting the file.
    pub fn delete(name: &str, presets_dir: &Path) -> Result<()> {
        fs::remove_file(presets_dir.join(name))?;
        Ok(())
    }

    /// Add a mod to the preset.
    ///
    /// # Arguments
    ///
    /// `mod_name`: The name of the mod to add.
    pub fn add_mod(&mut self, mod_name: &str) {
        self.mods.push(String::from(mod_name))
    }

    /// Add multiple mods to the preset.
    ///
    /// # Arguments
    ///
    /// `mods`: The mods to add.
    pub fn add_mods(&mut self, mods: &[String]) {
        self.mods.extend(mods.iter().cloned())
    }

    /// Remove a mod from the preset.
    ///
    /// Does nothing if the mod isn't in the preset. If the mod is in the preset multiple times,
    /// it removes every one. Duplicate mods is redundant anyway.
    ///
    /// # Arguments
    ///
    /// `mod_name`: The name of the mod to remove.
    pub fn remove_mod(&mut self, mod_name: &str) {
        self.mods.retain(|m| m != mod_name)
    }

    /// Remove multiple mods from the preset.
    ///
    /// Does nothing if any mods aren't in the preset. If a mod is in the preset multiple times,
    /// it removes every one. Duplicate mods is redundant anyway.
    ///
    /// # Arguments
    ///
    /// `mods`: The mods to remove.
    pub fn remove_mods(&mut self, mods: &[String]) {
        // Convert to HashSet so we can O(1) check if a mod is in the mods to remove.
        let values_to_remove: HashSet<&String> = mods.iter().collect();

        self.mods.retain(|m| !values_to_remove.contains(m))
    }

    /// Enable the preset.
    ///
    /// This method is NOT simply fire and forget. It will set this preset as enabled and nothing
    /// more. In order to actually enable the mods in this preset, the following steps must be
    /// taken:
    ///
    /// 1. Call `Preset::enable` on the preset.
    /// 2. Save the preset to the proper presets directory.
    /// 3. Call `ModCfg::apply_presets` on the ModCfg to enable the mods in memory.
    /// 4. Save the ModCfg to the proper mods directory, allowing the game to read the changes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beam_mm::{Preset, game::ModCfg};
    /// # use tempfile::tempdir;
    ///
    /// # // Set up temp mock directories
    /// # let temp_presets_dir = tempdir().unwrap();
    /// # let presets_dir = temp_presets_dir.path();
    /// # let temp_mods_dir = tempdir().unwrap();
    /// # let mods_dir = temp_mods_dir.path();
    /// # // Make mods_dir/db.json
    /// # std::fs::write(mods_dir.join("db.json"), "{\"mods\":{\"mod1\":{\"active\":false},\"mod2\":{\"active\":false}}}").unwrap();
    /// #
    /// let mut mod_cfg = ModCfg::load_from_path(&mods_dir).unwrap();
    /// let mut preset = Preset::new("preset_name".into(), vec!["mod1".into(), "mod2".into()]);
    ///
    /// preset.enable();
    /// preset.save_to_path(&presets_dir).unwrap();
    ///
    /// mod_cfg.apply_presets(&presets_dir).unwrap();
    /// mod_cfg.save_to_path(&mods_dir).unwrap();
    /// ```
    pub fn enable(&mut self) {
        self.enabled = true
    }

    /// Disable the preset.
    ///
    /// Similarly to `Preset::enable`, this method is NOT simply fire and forget. It will set this
    /// preset as disabled after modifying the ModCfg in memory. To actually disable the mods in
    /// this preset, the following steps must be taken:
    ///
    /// 1. Call `Preset::disable` on the preset.
    /// 2. Save the preset to the proper presets directory.
    /// 3. Call `ModCfg::apply_presets` on the ModCfg to enable the mods in memory for ENABLED
    ///    presets.
    /// 4. Save the ModCfg to the proper mods directory, allowing the game to read the changes.
    ///
    /// Calling this function does IMMEDIATELY disable the mods in the preset in memory. The reason
    /// this disables mods but still needs to be saved and applied is because the ModCfg needs to
    /// be able to re-enable any mods that are in other enabled presets.
    ///
    /// # Errors
    ///
    /// MissingMods: If one or more mods in the preset doesn't exist in the ModCfg.
    ///
    /// In case of error, ModCfg and this preset will remain unchanged.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beam_mm::{Preset, game::ModCfg};
    /// # use tempfile::tempdir;
    ///
    /// # // Set up temp mock directories
    /// # let temp_presets_dir = tempdir().unwrap();
    /// # let presets_dir = temp_presets_dir.path();
    /// # let temp_mods_dir = tempdir().unwrap();
    /// # let mods_dir = temp_mods_dir.path();
    /// # // Make mods_dir/db.json
    /// # std::fs::write(mods_dir.join("db.json"), "{\"mods\":{\"mod1\":{\"active\":true},\"mod2\":{\"active\":true}}}").unwrap();
    /// #
    /// let mut mod_cfg = ModCfg::load_from_path(&mods_dir).unwrap();
    /// let mut preset = Preset::new("preset_name".into(), vec!["mod1".into(), "mod2".into()]);
    ///
    /// preset.disable(&mut mod_cfg).unwrap();
    /// preset.save_to_path(&presets_dir).unwrap();
    ///
    /// mod_cfg.apply_presets(&presets_dir).unwrap();
    /// mod_cfg.save_to_path(&mods_dir).unwrap();
    /// ```
    pub fn disable(&mut self, mod_config: &mut ModCfg) -> Result<()> {
        mod_config.set_mods_active(&self.mods, false)?;
        self.enabled = false;
        Ok(())
    }

    /// Get the enabled status of the preset.
    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    /// Get a list of mods in the preset.
    pub fn get_mods(&self) -> &Vec<String> {
        &self.mods
    }
}
