use crate::*;
use tempfile::tempdir;

/// Struct to handle mock dirs for testing.
/// Automatically cleans up the directories when dropped.
/// _temp fields are the TempDir structs which need to be preserved so the directories are not
/// deleted before the tests are run.
pub struct MockData {
    _mods_dir_temp: tempfile::TempDir,
    pub mods_dir: PathBuf,
    _presets_dir_temp: tempfile::TempDir,
    pub presets_dir: PathBuf,
    pub modcfg: game::ModCfg,
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

        let modcfg = game::ModCfg::load_from_path(&mods_dir).unwrap();

        Self {
            _mods_dir_temp,
            mods_dir,
            _presets_dir_temp,
            presets_dir,
            modcfg,
        }
    }

    fn create_db_json(dir: &Path) {
        // NOTE: Changing this JSON could break tests.
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
}
