use crate::{Error::*, Result};
use dirs;
use std::{
    fs::{self},
    path::{Path, PathBuf},
};

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
///     creating the directory.
fn validate_dir(dir: PathBuf) -> Result<PathBuf> {
    if dir.try_exists()? {
        Ok(dir)
    } else {
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }
}

/// Get the path to the BeamNG.drive data directory if it exists.
///
/// # Arguments
///
/// * `custom_dir`: Optionally specify a custom directory where BeamNG holds its data. It will be
///     checked to make sure it exists; if it does not, `Err(SpecDirNotExists)` will be returned.
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
            .find(|d| d.try_exists().unwrap_or(false)) // Find the first existing path.
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
/// use beam_mm::path::mods_dir;
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
///     an issue creating the dir
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
///     is an issue creating the dir
///
/// # Examples
///
/// ```rust
/// use beam_mm::path::presets_dir;
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
