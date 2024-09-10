use dirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
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
    #[error("Could not find preset {preset} in {dir}")]
    MissingPreset { dir: PathBuf, preset: String },
    /// When mods are specified but not found.
    #[error("Mods not found: {mods:?}")]
    MissingMods { mods: Vec<String> },

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
/// Errors only on filesystem errors.
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

#[derive(Serialize, Deserialize)]
pub struct Preset {
    name: String,
    mods: Vec<String>,
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
    /// Possible IO errors.
    pub fn list(presets_dir: &Path) -> Result<impl Iterator<Item = String>> {
        Ok(fs::read_dir(presets_dir)?
            .filter_map(|f| f.ok().map(|f| f.path())) // Get rid of errors and map to path type
            .filter(|f| f.is_file() && f.extension().unwrap_or(OsStr::new("")) == "json") // Filter out dirs and non-json files
            // Map to remove the json extension so we just have the preset name and convert to String
            // if the os string into_string fails, it gets converted to None which gets filtered out
            .filter_map(|f| f.with_extension("").into_os_string().into_string().ok()))
    }

    pub fn new(name: String, mods: Vec<String>) -> Self {
        Preset {
            name,
            mods,
            enabled: false,
        }
    }

    pub fn save<W: Write>(&self, mut writer: W) -> Result<()> {
        serde_json::to_writer_pretty(&mut writer, self)?;
        writer.flush()?;

        Ok(())
    }

    pub fn save_to_path(&self, presets_dir: &Path) -> Result<()> {
        let file = File::create(presets_dir.join(&self.name))?;
        let writer = BufWriter::new(file);
        self.save(writer)
    }

    pub fn load<R: BufRead>(reader: R) -> Result<Self> {
        Ok(serde_json::from_reader(reader)?)
    }

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

    pub fn delete(name: &str, presets_dir: &Path) -> Result<()> {
        fs::remove_file(presets_dir.join(name))?;
        Ok(())
    }

    pub fn add_mod(&mut self, mod_name: &str) {
        self.mods.push(String::from(mod_name))
    }

    pub fn add_mods(&mut self, mods: &[String]) {
        self.mods.extend(mods.iter().cloned())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModCfg {
    mods: HashMap<String, Mod>,

    /// Data that is unimportant to us.
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

impl ModCfg {
    fn filename() -> PathBuf {
        PathBuf::from("db.json")
    }

    pub fn load<R: BufRead>(reader: R) -> Result<Self> {
        Ok(serde_json::from_reader(reader)?)
    }

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

    pub fn save<W: Write>(&self, mut writer: W) -> Result<()> {
        serde_json::to_writer_pretty(&mut writer, self)?;
        writer.flush()?;

        Ok(())
    }

    pub fn save_to_path(&self, mods_dir: &Path) -> Result<()> {
        let file = File::create(mods_dir.join(Self::filename()))?;
        let writer = BufWriter::new(file);
        self.save(writer)
    }

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

    // This function needs to only change self if everything is successful. If even one mod fails
    // somewhere, self should be returned unchanged.
    pub fn set_mods_active(&mut self, mod_names: &[String], active: bool) -> Result<()> {
        // First validate mods. If all exist, then we will push
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

    pub fn get_mods(&self) -> impl Iterator<Item = &String> {
        self.mods.keys()
    }

    pub fn set_all_mods_active(&mut self, active: bool) -> Result<()> {
        let mods: Vec<String> = self.get_mods().cloned().collect();
        self.set_mods_active(&mods, active)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Mod {
    // There is not yet a reason to give external access to Mod so keep private for now
    active: bool,

    /// Other unimportant data.
    #[serde(flatten)]
    other: HashMap<String, Value>,
}
