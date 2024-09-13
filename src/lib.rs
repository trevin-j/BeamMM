use dirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

/// Result type alias for this crate.
pub type Result<T> = core::result::Result<T, Error>;

/// Error enum for this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// When the specified directory does not exist.
    ///
    /// # Fields
    ///
    /// * `dir`: The directory that was specified but doesn't exist.
    #[error("Directory {dir} not found.")]
    DirNotFound { dir: PathBuf },
    /// When the game directory cannot be automatically found. Try launching the game first.
    #[error("Game directory could not automatically be found. Try launching the game first.")]
    GameDirNotFound,
    /// When `%LocalAppData%` Windows variable isn't found. What's wrong with your Windows install?
    #[error("%LocalAppData% variable could not be found.")]
    MissingLocalAppdata,
    /// When `version.txt` format is for some reason wrong.
    #[error("Could not parse BeamNG.drive's version.txt for game version.")]
    VersionError,
    /// When the preset wasn't found.
    ///
    /// # Fields
    ///
    /// * `dir`: The directory where the preset was supposed to be.
    /// * `preset`: The name of the preset that was missing.
    #[error("Could not find preset {preset} in {dir}")]
    MissingPreset { dir: PathBuf, preset: String },
    /// When mods are specified but not found.
    ///
    /// # Fields
    ///
    /// * `mods`: The mods that were specified but not found.
    #[error("Mods not found: {mods:?}")]
    MissingMods { mods: Vec<String> },
    // When a preset errors when enabling
    //
    // # Fields
    //
    // * `mods`: The mods that were missing.
    // * `presets`: The presets that failed to enable.
    #[error("Presets failed to enable: {presets:?}, missing these mods: {mods:?}")]
    PresetsFailed {
        mods: HashSet<String>,
        presets: HashSet<String>,
    },

    /// std::io errors.
    #[error("There was an IO error. {0}")]
    IO(#[from] std::io::Error),

    /// serder_json errors.
    #[error("There was a JSON error. {0}")]
    JSON(#[from] serde_json::Error),
}

use Error::*;

/// Check if a directory exists and create it if it doesn't. Consumes and returns the directory,
/// making it simple to use at the end of a function.
///
/// # Arguments
///
/// * `dir`: The directory to check and create if it doesn't exist.
///
/// # Errors
///
/// * `std::io::Error`: If there is a permission issue when checking if the directory exists or
/// creating the directory.
fn validate_dir(dir: PathBuf) -> Result<PathBuf> {
    if dir.try_exists()? {
        Ok(dir)
    } else {
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }
}

/// Get the game's major.minor version e.g. `0.32`.
///
/// # Arguments
///
/// * `data_dir`: The game's data directory. Usually `%LocalAppData%/BeamNG.Drive`. Can be found
/// using `beam_mm::beamng_dir(dir)`
///
/// # Errors
///
/// * `VersionError`:
///   * If the `version.txt` file exists but there is an issue with parsing the version
///   major.minor.
///   * If there is no `version.txt` and there is trouble manually discovering the version based on
///   the existing game version directories.
/// * `DirNotFound`: if the specified `data_dir` doesn't exist.
/// * `std::io::Error`: if there is trouble checking file existence or reading dir. Most likely due
/// to permission issues.
///
/// # Examples
///
/// ```rust
/// use beam_mm::game_version;
/// # use tempfile::tempdir;
///
/// # let temp_dir = tempdir().unwrap();
/// # let game_dir = temp_dir.path();
/// # let version_file = game_dir.join("version.txt");
/// # std::fs::write(&version_file, "0.32.0").unwrap();
/// // Game dir should be the path to the base game data directory.
/// // Most likely `%LocalAppData%/BeamNG.drive`
/// let version = game_version(&game_dir).unwrap();
/// ```
pub fn game_version(data_dir: &Path) -> Result<String> {
    if !data_dir.try_exists()? {
        return Err(DirNotFound {
            dir: data_dir.to_owned(),
        });
    }
    let version_path = data_dir.join("version.txt");
    if version_path.try_exists()? {
        // If the version.txt file exists in the data_dir, we can just read it to find the game
        // version.
        let full_version = fs::read_to_string(version_path)?;
        let mut split_version = full_version.trim().split(".");
        let major_version = split_version.next().ok_or(VersionError)?;
        let minor_version = split_version.next().ok_or(VersionError)?;
        Ok(format!("{},{}", major_version, minor_version))
    } else {
        // If there is no version.txt, a fallback is to list all the version directories and find
        // the latest one, assuming it is correct.
        fs::read_dir(data_dir)?
            .filter_map(|f| f.ok().map(|f| f.path())) // Unwrap all, tossing out any files/dirs that errored.
            .filter(|f| f.is_dir()) // Toss out non-dirs.
            .filter_map(
                |d| {
                    d.to_str() // Convert dir name to str.
                        .map(|d| d.parse::<f32>()) // Parse dir name to float (for version number).
                        .filter(|n| n.is_ok()) // Toss out dirs that failed to convert to float.
                        .map(|n| n.unwrap())
                }, // Safe to unwrap now that we know each value is Ok.
            )
            .reduce(f32::max) // Grab max version number
            .map(|n| n.to_string()) // Map version back to string
            .ok_or(VersionError) // If something went wrong and thus we can't find the version then error
    }
}

/// Get the path to the BeamNG.drive data directory if it exists.
///
/// # Arguments
///
/// * `custom_dir`: Optionally specify a custom directory where BeamNG holds its data. It will be
/// checked to make sure it exists; if it does not, `Err(SpecDirNotExists)` will be returned.
///
/// # Errors
///
/// * `DirNotFound`: When a custom directory is specified but it doesn't exist.
/// * `GameDirNotFound`: When the game's data directory cannot be found automatically.
pub fn beamng_dir(custom_dir: &Option<PathBuf>) -> Result<PathBuf> {
    if let Some(custom_dir) = custom_dir {
        if custom_dir.try_exists()? {
            Ok(custom_dir.to_owned())
        } else {
            Err(DirNotFound {
                dir: custom_dir.to_owned(),
            })
        }
    } else {
        vec![dirs::data_local_dir(), dirs::data_dir()] // Possible data dirs to look for game dir in
            .into_iter()
            .filter_map(|d| d.map(|d| d.join("BeamNG.drive"))) // Filter None, unwrap, and concat "BeamNG.drive" to path.
            .filter(|d| d.try_exists().unwrap_or(false)) // Filter out non-existing paths.
            .next() // Grab the first directory - most likely the only directory.
            .ok_or(GameDirNotFound {})
    }
}

/// Get the BeamNG.drive mods folder based on the game's base data dir and the game's version.
///
/// # Arguments
///
/// `data_dir`: The base game data directory. Usually `%LocalAppData%/BeamNG.drive`
/// `version`: The current game version. Can be retrieved via `beam_mm::game_version(data_dir)`.
///
/// # Errors
///
/// `DirNotFound`: When passed in data_dir doesn't exist or the mods dir under the current version
/// dir doesn't exist. Try launching the game first?
/// `std::io::Error`: If there is a permission error in checking the existence of any dirs.
///
/// # Examples
///
/// ```rust
/// use beam_mm::mods_dir;
/// # use tempfile::tempdir;
///
/// # let temp_dir = tempdir().unwrap();
/// # let data_dir = temp_dir.path();
/// # let version = "0.32";
/// # std::fs::create_dir_all(data_dir.join(version).join("mods")).unwrap();
/// let mods_dir = mods_dir(&data_dir, &version).unwrap();
/// ```
pub fn mods_dir(data_dir: &Path, version: &str) -> Result<PathBuf> {
    // Confirm data_dir even exists.
    if !data_dir.try_exists()? {
        Err(DirNotFound {
            dir: data_dir.to_owned(),
        })
    } else {
        // Find the mods_dir. To do this, we need to find the game version, enter that version.
        // folder, and return the mods dir inside that folder after verifying it exists.
        let mods_dir_ = data_dir.join(version).join("mods");
        if mods_dir_.try_exists()? {
            Ok(mods_dir_)
        } else {
            Err(DirNotFound { dir: mods_dir_ })
        }
    }
}

/// Get the path to the beammm directory and create it if it doesn't exist.
///
/// # Errors
///
/// * `MissingLocalAppdata` if there is a problem retrieving the `%LocalAppData%` Windows variable
/// * `std::io::Error` if there is a permissions issue when checking if the dir exists or if there is
/// an issue creating the dir
pub fn beammm_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or(MissingLocalAppdata)?
        .join("BeamMM");

    validate_dir(dir)
}

/// Get the path to the presets directory and create it if it doesn't exist.
///
/// # Arguments
///
/// `beammm_dir`: The path to the beammm directory.
///
/// # Errors
///
/// * `std::io::Error` if there is a permissions issue when checking if the dir exists or if there
/// is an issue creating the dir
///
/// # Examples
///
/// ```rust
/// use beam_mm::presets_dir;
/// # use tempfile::tempdir;
///
/// # let temp_dir = tempdir().unwrap();
/// # let beammm_dir = temp_dir.path();
/// let presets_dir = presets_dir(&beammm_dir).unwrap();
/// ```
pub fn presets_dir(beammm_dir: &Path) -> Result<PathBuf> {
    let dir = beammm_dir.join("presets");
    validate_dir(dir)
}

/// Confirm a choice with the user.
///
/// For testability, this function requires a BufRead and Write to do reading and writing. For a
/// simple convenience wrapper around this that uses stdio, use `confirm_cli`.
///
/// # Arguments
///
/// `reader`: Thing to read from e.g. stdin.
/// `writer`: Thing to write to e.g. stdout.
/// `msg`: The confirmation message to display to the user.
/// `default`: The default choice.
/// `confirm_all`: Whether or not to confirm all.
///
/// # Errors
///
/// IO errors are possible from read and write operations.
pub fn confirm<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    msg: &str,
    default: bool,
    confirm_all: bool,
) -> Result<bool> {
    if confirm_all {
        Ok(true)
    } else {
        let y_n = String::from(if default { "(Y/n)" } else { "(y/N)" });

        write!(&mut writer, "{} {}", msg.trim(), y_n)?;

        let mut input = String::new();
        reader.read_line(&mut input)?;

        input = input.trim().to_lowercase();

        if default {
            Ok(input != "n")
        } else {
            Ok(input == "y")
        }
    }
}

/// Convenience function that wraps the `confirm` function with stdio. Confirm a choice with the user.
///
/// # Arguments
///
/// `msg`: The confirmation message to display to the user.
/// `default`: The default choice.
/// `confirm_all`: Whether or not to confirm all.
///
/// # Errors
///
/// IO errors are possible from read and write operations.
pub fn confirm_cli(msg: &str, default: bool, confirm_all: bool) -> Result<bool> {
    confirm(io::stdin().lock(), io::stdout(), msg, default, confirm_all)
}

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
    /// use beam_mm::{Preset, ModCfg};
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
    /// use beam_mm::{Preset, ModCfg};
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

/// A struct representing BeamNG.drive's mod configuration.
///
/// This struct is used to load, modify, and save the game's mod configuration.
#[derive(Serialize, Deserialize, Debug)]
pub struct ModCfg {
    /// Installed mods and their data.
    mods: HashMap<String, Mod>,

    /// Additional data that is currently unimportant to us but should be preserved.
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

impl ModCfg {
    /// The filename of the mod configuration file.
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
    /// use beam_mm::{Preset, ModCfg};
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
            if preset.get_enabled() {
                match self.set_mods_active(&preset.mods, true) {
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

        if failed_presets.len() > 0 {
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

        if missing_mods.len() > 0 {
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
}

/// A struct representing a BeamNG.drive mod.
#[derive(Serialize, Deserialize, Debug)]
struct Mod {
    /// Whether the mod is active.
    active: bool,

    /// Other currently unimportant data.
    #[serde(flatten)]
    other: HashMap<String, Value>,
}
