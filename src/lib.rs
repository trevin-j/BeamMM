use derive_more::From;
use dirs;
use std::{
    ffi::OsStr,
    fs,
    io::{self, BufRead, Write},
    path::PathBuf,
};

/// Result type alias for this crate.
pub type Result<T> = core::result::Result<T, Error>;

/// Error enum for this crate.
#[derive(Debug, From)]
pub enum Error {
    /// When the specified directory does not exist.
    ///
    /// # Fields
    ///
    /// * `dir`: The directory that was specified but doesn't exist.
    DirNotFound { dir: PathBuf },
    /// When the game directory cannot be automatically found. Try launching the game first.
    GameDirNotFound,
    /// When `%LocalAppData%` Windows variable isn't found. What's wrong with your Windows install?
    MissingLocalAppdata,
    /// When `version.txt` format is for some reason wrong.
    VersionError,

    /// std::io errors.
    #[from]
    IOError(std::io::Error),
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
pub fn game_version(data_dir: &PathBuf) -> Result<String> {
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
pub fn mods_dir(data_dir: &PathBuf, version: &String) -> Result<PathBuf> {
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

pub fn presets_dir(beammm_dir: &PathBuf) -> Result<PathBuf> {
    let dir = beammm_dir.join("presets");
    validate_dir(dir)
}

/// Get an iterator over currently saved presets.
///
/// # Arguments
///
/// `presets_dir`: Where preset config files are stored.
///
/// # Errors
///
/// Possible IO errors.
pub fn get_presets(presets_dir: &PathBuf) -> Result<impl Iterator<Item = PathBuf>> {
    Ok(fs::read_dir(presets_dir)?
        .filter_map(|f| f.ok().map(|f| f.path())) // Get rid of errors and map to path type
        .filter(|f| f.is_file() && f.extension().unwrap_or(OsStr::new("")) == "json") // Filter out dirs and non-json files
        .map(|f| f.with_extension(""))) // Map to remove the json extension so we just have the preset name
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
