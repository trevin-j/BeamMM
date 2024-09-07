use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// BeamMM CLI - A mod manager backend and command line application for the game BeamNG.Drive
struct Args {
    /// Add mods to be tracked by BeamMM - does not install; use --update all after adding to
    /// install
    #[arg(short, long)]
    add: bool,

    /// Remove/uninstall mods - pass "all" to remove all mods
    #[arg(short, long)]
    remove: bool,

    /// Update the specified mods - pass "all" to update all mods
    #[arg(short, long)]
    update: bool,

    /// Create a mod preset
    #[arg(long, value_name = "PRESET")]
    create_preset: Option<String>,

    /// Permanently delete a preset
    #[arg(long, value_name = "PRESET")]
    delete_preset: Option<String>,

    /// Add mods to a preset
    #[arg(long, value_name = "PRESET")]
    preset_add: Option<String>,

    /// Remove mods from a preset
    #[arg(long, value_name = "PRESET")]
    preset_remove: Option<String>,

    /// Select the mods for the chosen operation
    mods: Option<Vec<String>>,

    /// Disable mods - pass "all" to disable all mods
    #[arg(long)]
    disable: bool,

    /// Enable mods - pass "all" to enable all mods
    #[arg(long)]
    enable: bool,

    /// Enable a preset
    #[arg(long, value_name = "PRESET")]
    enable_preset: Option<String>,

    /// Disable a preset
    #[arg(long, value_name = "PRESET")]
    disable_preset: Option<String>,

    /// Answer yes to all confirmation prompts
    #[arg(long, short = 'y')]
    confirm_all: bool,

    /// Choose a custom BeamNG data directory
    #[arg(long, value_name = "DIR")]
    custom_data_dir: Option<PathBuf>,
}

fn main() -> beam_mm::Result<()> {
    let args = Args::parse();

    let beamng_dir = beam_mm::beamng_dir(args.custom_data_dir)?;
    let beamng_version = beam_mm::game_version(beamng_dir)?;
    let mods_dir = beam_mm::mods_dir(beamng_dir, beamng_version)?;
    let beammm_dir = beam_mm::beammm_dir()?;

    if let Some(preset) = args.create_preset {
        beam_mm::create_preset(preset, args.mods.unwrap_or(vec![]));
    }
    if let Some(preset) = args.delete_preset {
        let confirmation = beam_mm::confirm(
            format!("Are you sure you want to delete preset '{}'?", preset),
            false,
            args.confirm_all,
        );
        if confirmation {
            beam_mm::delete_preset(preset);
        }
    }
    if let Some(preset) = args.enable_preset {
        beam_mm::enable_preset(preset);
    }
    if let Some(preset) = args.disable_preset {
        beam_mm::disable_preset(preset);
    }

    // Handle operations that require args.mods to exist.
    if let Some(mods) = args.mods {
        // Check of mods argument is "all"
        let all_mods = Some(String::from("all")) == mods.get(0).map(|s| s.to_lowercase());

        if args.add {
            beam_mm::add_mods(mods)?;
        }
        if args.remove {
            if all_mods {
                let confirmation = beam_mm::confirm(
                    "Are you sure you would like to remove all mods?".into(),
                    false,
                    args.confirm_all,
                );
                if confirmation {
                    beam_mm::remove_all_mods()?;
                }
            } else {
                beam_mm::remove_mods(mods)?;
            }
        }
        if args.update {
            if all_mods {
                beam_mm::update_all_mods()?;
            } else {
                beam_mm::update_mods(mods)?;
            }
        }
        if args.enable {
            if all_mods {
                let confirmation = beam_mm::confirm(
                    "Are you sure you would like to enable all mods?".into(),
                    true,
                    args.confirm_all,
                );
                if confirmation {
                    beam_mm::enable_all_mods()?;
                }
            } else {
                beam_mm::enable_mods(mods);
            }
        }
        if args.disable {
            if all_mods {
                let confirmation = beam_mm::confirm(
                    "Are you sure you would like to disable all mods?".into(),
                    false,
                    args.confirm_all,
                );
                if confirmation {
                    beam_mm::disable_all_mods()?;
                }
            } else {
                beam_mm::disable_mods(mods);
            }
        }
        if let Some(preset) = args.preset_add {
            beam_mm::add_to_preset(preset, mods);
        }
        if let Some(preset) = args.preset_remove {
            beam_mm::remove_from_preset(preset, mods);
        }
    }

    Ok(())
}
