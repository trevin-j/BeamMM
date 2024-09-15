use std::{
    collections::HashSet,
    fs::{self},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

pub mod game;
pub mod path;
mod preset;

pub use preset::Preset;

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

/// Get the game's major.minor version e.g. `0.32`.
///
/// # Arguments
///
/// * `data_dir`: The game's data directory. Usually `%LocalAppData%/BeamNG.Drive`. Can be found
///     using `beam_mm::beamng_dir(dir)`
///
/// # Errors
///
/// * `VersionError`:
///     * If the `version.txt` file exists but there is an issue with parsing the version
///         major.minor.
///     * If there is no `version.txt` and there is trouble manually discovering the version based on
///         the existing game version directories.
/// * `DirNotFound`: if the specified `data_dir` doesn't exist.
/// * `std::io::Error`: if there is trouble checking file existence or reading dir. Most likely due
///     to permission issues.
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
        Ok(format!("{}.{}", major_version, minor_version))
    } else {
        // If there is no version.txt, a fallback is to list all the version directories and find
        // the latest one, assuming it is correct.
        fs::read_dir(data_dir)?
            .filter_map(|f| f.ok().map(|f| f.path())) // Unwrap all, tossing out any files/dirs that errored.
            .filter(|f| f.is_dir()) // Toss out non-dirs.
            .filter_map(
                |d| {
                    d.file_name()
                        .and_then(|d| d.to_str()) // Convert dir name to str.
                        .map(|d| d.trim().parse::<f32>()) // Parse dir name to float (for version number).
                        .filter(|n| n.is_ok()) // Toss out dirs that failed to convert to float.
                        .map(|n| n.unwrap())
                }, // Safe to unwrap now that we know each value is Ok.
            )
            .reduce(f32::max) // Grab max version number
            .map(|n| n.to_string()) // Map version back to string
            .ok_or(VersionError) // If something went wrong and thus we can't find the version then error
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_confirm() {
        // We need to test the following situations:
        // 1. Default is true, user inputs "n" -> false
        // 2. Default is true, user inputs "y" -> true
        // 3. Default is true, user inputs nothing -> true
        // 4. Default is false, user inputs "n" -> false
        // 5. Default is false, user inputs "y" -> true
        // 6. Default is false, user inputs nothing -> false

        let input_y = b"y\n";
        let input_n = b"n\n";
        let input_nothing = b"\n";

        let mut writer = Vec::new();

        let msg = "Are you sure?";
        let confirm_all = false;

        {
            let mut reader_n = io::BufReader::new(&input_n[..]);
            let result = confirm(&mut reader_n, &mut writer, msg, true, confirm_all).unwrap();
            assert!(!result);
        }
        {
            let mut reader_y = io::BufReader::new(&input_y[..]);
            let result = confirm(&mut reader_y, &mut writer, msg, true, confirm_all).unwrap();
            assert!(result);
        }
        {
            let mut reader_nothing = io::BufReader::new(&input_nothing[..]);
            let result = confirm(&mut reader_nothing, &mut writer, msg, true, confirm_all).unwrap();
            assert!(result);
        }
        {
            let mut reader_n = io::BufReader::new(&input_n[..]);
            let result = confirm(&mut reader_n, &mut writer, msg, false, confirm_all).unwrap();
            assert!(!result);
        }
        {
            let mut reader_y = io::BufReader::new(&input_y[..]);
            let result = confirm(&mut reader_y, &mut writer, msg, false, confirm_all).unwrap();
            assert!(result);
        }
        {
            let mut reader_nothing = io::BufReader::new(&input_nothing[..]);
            let result =
                confirm(&mut reader_nothing, &mut writer, msg, false, confirm_all).unwrap();
            assert!(!result);
        }
        // If confirm_all is true, it should always return true.
        {
            let mut reader_n = io::BufReader::new(&input_n[..]);
            let result = confirm(&mut reader_n, &mut writer, msg, true, true).unwrap();
            assert!(result);
        }
    }

    #[test]
    fn test_game_version() {
        let temp_dir = tempdir().unwrap();
        let game_dir = temp_dir.path();
        let version_file = game_dir.join("version.txt");

        std::fs::write(&version_file, "0.32.0").unwrap();

        let version = game_version(game_dir).unwrap();

        assert_eq!(version, "0.32");
    }

    /// Discover the game version based on the folders in the game data directory.
    #[test]
    fn test_discover_game_version() {
        let temp_dir = tempdir().unwrap();
        let game_dir = temp_dir.path();
        // Make a few directories with version numbers.
        std::fs::create_dir(game_dir.join("0.31")).unwrap();
        std::fs::create_dir(game_dir.join("0.32")).unwrap();
        std::fs::create_dir(game_dir.join("0.33")).unwrap();

        let version = game_version(game_dir).unwrap();

        assert_eq!(version, "0.33");
    }

    #[test]
    fn test_game_version_bad_directory() {
        let game_dir = Path::new("nonexistent");

        let version = game_version(game_dir);

        assert!(matches!(version, Err(DirNotFound { .. })));
    }
}
