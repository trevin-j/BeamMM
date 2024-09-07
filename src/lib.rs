use derive_more::From;
use dirs;
use std::{fs, path::PathBuf};

/// Result type alias for this crate.
pub type Result<T> = core::result::Result<T, Error>;

/// Error enum for this crate.
#[derive(Debug, From)]
pub enum Error {
    /// When the specified directory does not exist.
    ///
    /// # Fields
    ///
    /// * dir - The directory that was specified but doesn't exist.
    SpecDirNotExists { dir: PathBuf },
    /// When the game directory cannot be automatically found. Try launching the game first.
    GameDirNotFound,
    /// When %LocalAppData% Windows variable isn't found. What's wrong with your Windows install?
    MissingLocalAppdata,

    /// std::io errors.
    #[from]
    IOError(std::io::Error),
}

use Error::*;

/// Get the path to the BeamNG.drive data directory if it exists.
///
/// # Arguments
///
/// * mods_dir - Optionally specify a custom directory where BeamNG holds its data. It will be
/// checked to make sure it exists; if it does not, Err(SpecDirNotExists) will be returned.
///
/// # Errors
///
/// * SpecDirNotExists - When a custom directory is specified but it doesn't exist.
/// * GameDirNotFound - When the game's data directory cannot be found automatically.
pub fn beamng_dir(mods_dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(mods_dir_) = mods_dir {
        if mods_dir_.exists() {
            Ok(mods_dir_)
        } else {
            Err(SpecDirNotExists { dir: mods_dir_ })
        }
    } else {
        vec![dirs::data_local_dir(), dirs::data_dir()] // Possible data dirs to look for game dir in
            .into_iter()
            .filter_map(|d| d.map(|d| d.join("BeamNG.drive"))) // Filter None, unwrap, and concat "BeamNG.drive" to path
            .filter(|d| d.try_exists().unwrap_or(false)) // Filter out non-existing paths
            .next() // Grab the first directory - most likely the only directory
            .ok_or(GameDirNotFound {})
    }
}

/// Get the path to the beammm directory and create it if it doesn't exist
///
/// # Errors
///
/// * MissingLocalAppdata if there is a problem retrieving the %LocalAppData% Windows variable
/// * std::io::Error if there is a permissions issue when checking if the dir exists or if there is
/// an issue creating the dir
pub fn beammm_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or(MissingLocalAppdata)?
        .join("BeamMM");

    if !dir.try_exists()? {
        fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}
