use crate::{Error::*, Preset, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

/// A struct representing BeamNG.drive's mod configuration.
///
/// This struct is used to load, modify, and save the game's mod configuration.
#[derive(Serialize, Deserialize, Debug)]
pub struct ModCfg {
    /// Installed mods and their data.
    mods: HashMap<String, Mod>,

    /// Additional data that is currently unimportant to us but should be preserved.
    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
}

impl ModCfg {
    /// The filename of the mod configuration file.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn filename() -> PathBuf {
        PathBuf::from("db.json")
    }

    /// Load the mod configuration from a reader.
    ///
    /// # Arguments
    ///
    /// `reader`: The reader to load the mod configuration from.
    ///
    /// # Errors
    ///
    /// Possible serde_json errors if there is an issue reading or deserializing the mod
    /// configuration.
    pub fn load<R: BufRead>(reader: R) -> Result<Self> {
        Ok(serde_json::from_reader(reader)?)
    }

    /// Load the mod configuration from a file.
    ///
    /// # Arguments
    ///
    /// `mods_dir`: The directory where the mod configuration file is stored.
    ///
    /// # Errors
    ///
    /// Possible IO errors if there is an issue reading the file or serde_json errors if there is
    /// an issue deserializing the mod configuration.
    pub fn load_from_path(mods_dir: &Path) -> Result<Self> {
        if mods_dir.try_exists()? {
            let file = File::open(mods_dir.join(Self::filename()))?;
            let reader = BufReader::new(file);
            Self::load(reader)
        } else {
            Err(DirNotFound {
                dir: mods_dir.into(),
            })
        }
    }

    /// Apply all enabled presets in the presets directory.
    ///
    /// If a preset errors for any reason when enabling, said preset's mods will NOT be
    /// enabled. Any successfully enabled presets will have its mods fully enabled regardless of
    /// other presets erroring.
    ///
    /// # Arguments
    ///
    /// `presets_dir`: The directory where the presets are stored.
    ///
    /// # Errors
    ///
    /// MissingMods: If one or more mods in a preset doesn't exist in the ModCfg.
    /// PresetsFailed: If one or more presets failed to enable due to missing mods.
    /// Other errors: If there is an IO error when reading the presets directory or if there is an
    /// issue serializing the presets.
    ///
    /// # Examples
    /// ```rust
    /// use beammm::{Preset, game::ModCfg};
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
    pub fn apply_presets(&mut self, presets_dir: &Path) -> Result<()> {
        let mut missing_mods = HashSet::new();
        let mut failed_presets = HashSet::new();

        for preset_name in Preset::list(presets_dir)? {
            let preset = Preset::load_from_path(&preset_name, presets_dir)?;
            if preset.is_enabled() {
                match self.set_mods_active(preset.get_mods(), true) {
                    Ok(()) => (),
                    Err(e) => match e {
                        MissingMods { mods } => {
                            missing_mods.extend(mods);
                            failed_presets.insert(preset_name);
                        }
                        other => return Err(other), // Should not happen
                    },
                }
            }
        }

        if !failed_presets.is_empty() {
            Err(PresetsFailed {
                mods: missing_mods,
                presets: failed_presets,
            })
        } else {
            Ok(())
        }
    }

    /// Serialize and save the mod configuration to a writer.
    ///
    /// # Arguments
    ///
    /// `writer`: The writer to save the mod configuration to.
    ///
    /// # Errors
    ///
    /// Possible serde_json errors if there is an issue serializing the mod configuration or
    /// writing.
    pub fn save<W: Write>(&self, mut writer: W) -> Result<()> {
        serde_json::to_writer_pretty(&mut writer, self)?;
        writer.flush()?;

        Ok(())
    }

    /// Serialize and save the mod configuration to a file.
    ///
    /// # Arguments
    ///
    /// `mods_dir`: The directory where the mod configuration file will be saved.
    ///
    /// # Errors
    ///
    /// Possible IO errors if there is an issue creating the file or writing to it.
    /// Possible serde_json errors if there is an issue serializing the mod configuration.
    pub fn save_to_path(&self, mods_dir: &Path) -> Result<()> {
        let file = File::create(mods_dir.join(Self::filename()))?;
        let writer = BufWriter::new(file);
        self.save(writer)
    }

    /// Set a mod to be active or inactive.
    ///
    /// # Arguments
    ///
    /// `mod_name`: The name of the mod to set active or inactive.
    /// `active`: Whether the mod should be active or inactive.
    ///
    /// # Errors
    ///
    /// MissingMods: If the mod doesn't exist in the ModCfg.
    pub fn set_mod_active(&mut self, mod_name: &str, active: bool) -> Result<()> {
        if let Some(mod_) = self.mods.get_mut(mod_name) {
            mod_.active = active;
            Ok(())
        } else {
            Err(MissingMods {
                mods: vec![mod_name.into()],
            })
        }
    }

    /// Set multiple mods to be active or inactive.
    ///
    /// If any mods don't exist in the ModCfg, no mods will be set active or inactive.
    ///
    /// # Arguments
    ///
    /// `mod_names`: The names of the mods to set active or inactive.
    /// `active`: Whether the mods should be active or inactive.
    ///
    /// # Errors
    ///
    /// MissingMods: If one or more mods don't exist in the ModCfg.
    pub fn set_mods_active(&mut self, mod_names: &[String], active: bool) -> Result<()> {
        // First validate mods. If all exist, then we will set them active.
        let mut missing_mods = vec![];
        for mod_name in mod_names {
            if !self.mods.contains_key(mod_name) {
                missing_mods.push(mod_name.clone());
            }
        }

        if !missing_mods.is_empty() {
            Err(MissingMods { mods: missing_mods })
        } else {
            for mod_name in mod_names {
                self.set_mod_active(mod_name, active).unwrap(); // We've checked that every mod exists.
                                                                // enable_mod can only error if a mod
                                                                // doesn't exist so this is safe.
            }
            Ok(())
        }
    }

    /// Get a list of mods in the mod configuration.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn get_mods(&self) -> impl Iterator<Item = &String> {
        self.mods.keys()
    }

    /// Set all mods to be active or inactive.
    ///
    /// # Arguments
    ///
    /// `active`: Whether the mods should be active or inactive.
    ///
    /// # Errors
    ///
    /// MissingMods: If one or more mods don't exist in the ModCfg.
    pub fn set_all_mods_active(&mut self, active: bool) -> Result<()> {
        let mods: Vec<String> = self.get_mods().cloned().collect();
        self.set_mods_active(&mods, active)
    }

    /// Get the active status of a mod.
    ///
    /// # Arguments
    ///
    /// `mod_name`: The name of the mod to check the active status of.
    ///
    /// # Returns
    ///
    /// `Some(bool)`: The active status of the mod if it exists.
    /// `None`: If the mod doesn't exist in the ModCfg.
    pub fn is_mod_active(&self, mod_name: &str) -> Option<bool> {
        self.mods.get(mod_name).map(|m| m.active)
    }
}

/// A struct representing a BeamNG.drive mod.
#[derive(Serialize, Deserialize, Debug)]
struct Mod {
    /// Whether the mod is active.
    active: bool,

    /// Other currently unimportant data.
    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::MockData;

    #[test]
    fn loading_modcfg() {
        let mock_dirs = MockData::new();

        // Load the modcfg here instead of just relying on the MockDirs struct so we can test the loading.
        let mod_cfg = ModCfg::load_from_path(&mock_dirs.mods_dir).unwrap();

        assert_eq!(mod_cfg.mods.len(), 3);
        assert!(mod_cfg.mods.get("mod1").unwrap().active);
        assert!(!mod_cfg.mods.get("mod2").unwrap().active);
        assert!(mod_cfg.mods.get("mod3").unwrap().active);

        // Check that the other data is preserved.
        assert_eq!(mod_cfg.other.len(), 1);
        let other_data = mod_cfg.other.get("other").unwrap();
        assert_eq!(
            other_data.get("key").unwrap(),
            &serde_json::Value::String("value".into())
        );
    }

    #[test]
    fn load_bad_path() {
        let tmp = tempfile::tempdir().unwrap();
        let temp_dir = tmp.path();

        let result = ModCfg::load_from_path(&temp_dir.join("bad_path"));
        assert!(matches!(result, Err(DirNotFound { .. })));
    }

    #[test]
    fn save_modcfg() {
        let mock_dirs = MockData::new();

        let mut mod_cfg = mock_dirs.modcfg;
        mod_cfg.mods.get_mut("mod1").unwrap().active = false;

        mod_cfg.save_to_path(&mock_dirs.mods_dir).unwrap();

        let mod_cfg = ModCfg::load_from_path(&mock_dirs.mods_dir).unwrap();

        assert!(!mod_cfg.mods.get("mod1").unwrap().active);
    }

    #[test]
    fn set_mod_active() {
        let mock_dirs = MockData::new();

        let mut mod_cfg = mock_dirs.modcfg;
        mod_cfg.set_mod_active("mod1", false).unwrap();

        assert!(!mod_cfg.mods.get("mod1").unwrap().active);

        mod_cfg.set_mod_active("mod1", true).unwrap();

        assert!(mod_cfg.mods.get("mod1").unwrap().active);
    }

    #[test]
    fn set_mod_active_missing() {
        let mock_dirs = MockData::new();

        let mut mod_cfg = mock_dirs.modcfg;

        let result = mod_cfg.set_mod_active("fake_mod", true);
        assert!(matches!(result, Err(MissingMods { .. })));
    }

    #[test]
    fn set_mods_active() {
        let mock_dirs = MockData::new();

        let mut mod_cfg = mock_dirs.modcfg;
        mod_cfg
            .set_mods_active(&["mod1".into(), "mod2".into()], false)
            .unwrap();

        assert!(!mod_cfg.mods.get("mod1").unwrap().active);
        assert!(!mod_cfg.mods.get("mod2").unwrap().active);

        mod_cfg
            .set_mods_active(&["mod1".into(), "mod2".into()], true)
            .unwrap();

        assert!(mod_cfg.mods.get("mod1").unwrap().active);
        assert!(mod_cfg.mods.get("mod2").unwrap().active);
    }

    #[test]
    fn set_mods_active_missing() {
        let mock_dirs = MockData::new();

        let mut mod_cfg = mock_dirs.modcfg;

        let result = mod_cfg.set_mods_active(&["mod1".into(), "fake_mod".into()], true);
        assert!(matches!(result, Err(MissingMods { .. })));

        // Check that no mods were set active.
        assert!(mod_cfg.mods.get("mod1").unwrap().active);
        assert!(!mod_cfg.mods.get("mod2").unwrap().active);
    }

    #[test]
    fn set_all_mods_active() {
        let mock_dirs = MockData::new();

        let mut mod_cfg = mock_dirs.modcfg;
        mod_cfg.set_all_mods_active(false).unwrap();

        assert!(!mod_cfg.mods.get("mod1").unwrap().active);
        assert!(!mod_cfg.mods.get("mod2").unwrap().active);

        mod_cfg.set_all_mods_active(true).unwrap();

        assert!(mod_cfg.mods.get("mod1").unwrap().active);
        assert!(mod_cfg.mods.get("mod2").unwrap().active);
    }

    #[test]
    fn is_mod_active() {
        let mock_dirs = MockData::new();

        let mod_cfg = mock_dirs.modcfg;

        assert!(mod_cfg.is_mod_active("mod1").unwrap());
        assert!(!mod_cfg.is_mod_active("mod2").unwrap());
        assert!(mod_cfg.is_mod_active("mod3").unwrap());
        assert!(mod_cfg.is_mod_active("fake_mod").is_none());
    }

    #[test]
    fn apply_presets() {
        let mock_data = MockData::new();

        let mut preset1 = mock_data.preset1;
        let mut preset2 = mock_data.preset2;

        let mut mod_cfg = mock_data.modcfg;

        // Enable both presets and make sure both mods are enabled.

        preset1.enable();
        preset2.enable();

        preset1.save_to_path(&mock_data.presets_dir).unwrap();
        preset2.save_to_path(&mock_data.presets_dir).unwrap();

        mod_cfg.apply_presets(&mock_data.presets_dir).unwrap();

        assert!(mod_cfg.mods.get("mod1").unwrap().active);
        assert!(mod_cfg.mods.get("mod2").unwrap().active);

        // Disable just preset 2, which has both mod1 and mod2. Before applying preset, both mods
        // should be disabled. But, since preset 1 is still enabled, after applying preset, mod1
        // should be enabled.

        preset2.disable(&mut mod_cfg).unwrap();

        preset2.save_to_path(&mock_data.presets_dir).unwrap();

        assert!(!mod_cfg.mods.get("mod1").unwrap().active);
        assert!(!mod_cfg.mods.get("mod2").unwrap().active);

        mod_cfg.apply_presets(&mock_data.presets_dir).unwrap();

        assert!(mod_cfg.mods.get("mod1").unwrap().active);
        assert!(!mod_cfg.mods.get("mod2").unwrap().active);
    }

    #[test]
    fn apply_presets_missing_mods() {
        let mock_data = MockData::new();

        let mut preset1 = mock_data.preset1;
        let mut preset2 = mock_data.preset2;

        let mut mod_cfg = mock_data.modcfg;

        // Enable both presets and make sure both mods are enabled.

        preset1.enable();
        preset2.enable();

        preset1.save_to_path(&mock_data.presets_dir).unwrap();
        preset2.save_to_path(&mock_data.presets_dir).unwrap();

        // Remove mod2 from the modcfg so that preset2 will fail to enable.
        mod_cfg.mods.remove("mod2");

        let result = mod_cfg.apply_presets(&mock_data.presets_dir);
        assert!(matches!(result, Err(PresetsFailed { .. })));

        // Check that mod1 is still enabled.
        assert!(mod_cfg.mods.get("mod1").unwrap().active);
    }
}
