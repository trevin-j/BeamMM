#![cfg(test)]

use crate::*;
use tempfile::tempdir;

/// Struct to handle mock dirs for testing.
/// Automatically cleans up the directories when dropped.
/// _temp fields are the TempDir structs which need to be preserved so the directories are not
/// deleted before the tests are run.
/// Initializing a MockData makes multiple .unwrap() calls, so if tests start breaking as a result
/// of this, something is wrong with the code here. These calls shouldn't fail as they rely on the
/// temporary directories created by tempfile.
pub struct MockData {
    _mods_dir_temp: tempfile::TempDir,
    pub mods_dir: PathBuf,
    _presets_dir_temp: tempfile::TempDir,
    pub presets_dir: PathBuf,
    pub modcfg: game::ModCfg,
    pub preset1: Preset,
    pub preset2: Preset,
}

impl MockData {
    /// Initialize MockDirs with temporary directories.
    /// Creates a db.json file in the mods directory.
    pub fn new() -> Self {
        let _mods_dir_temp = tempdir().unwrap();
        let mods_dir = _mods_dir_temp.path().to_path_buf();
        let _presets_dir_temp = tempdir().unwrap();
        let presets_dir = _presets_dir_temp.path().to_path_buf();

        Self::create_db_json(&mods_dir);
        let (preset1, preset2) = Self::create_mock_presets(&presets_dir);

        let modcfg = game::ModCfg::load_from_path(&mods_dir).unwrap();

        Self {
            _mods_dir_temp,
            mods_dir,
            _presets_dir_temp,
            presets_dir,
            modcfg,
            preset1,
            preset2,
        }
    }

    fn create_db_json(dir: &Path) {
        // NOTE: Changing this JSON will most likely break some tests!
        let db_json = r#"{
            "mods": {
                "mod1": {
                    "active": true,
                    "other": {
                        "key": "value"
                    }
                },
                "mod2": {
                    "active": false,
                    "other": {
                        "key": "value"
                    }
                }
            },
            "other": {
                "key": "value"
            }
        }"#;

        std::fs::write(dir.join("db.json"), db_json).unwrap();
    }

    fn create_mock_presets(dir: &Path) -> (Preset, Preset) {
        // NOTE: Changing these JSONs will most likely break some tests!
        let preset1 = r#"{
            "name": "preset1",
            "mods": [
                "mod1"
            ],
            "enabled": true
        }"#;

        let preset2 = r#"{
            "name": "preset2",
            "mods": [
                "mod1",
                "mod2"
            ],
            "enabled": false
        }"#;

        std::fs::write(dir.join("preset1.json"), preset1).unwrap();
        std::fs::write(dir.join("preset2.json"), preset2).unwrap();

        (
            Preset::load_from_path("preset1", dir).unwrap(),
            Preset::load_from_path("preset2", dir).unwrap(),
        )
    }
}
