use beam_mm::path::*;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// BeamMM CLI - A mod manager backend and command line application for the game BeamNG.drive
struct Args {
    /// Create a mod preset
    #[arg(long, value_name = "NAME")]
    create_preset: Option<String>,

    /// Permanently delete a preset
    #[arg(long, value_name = "NAME")]
    delete_preset: Option<String>,

    /// Add mods to a preset
    #[arg(long, value_name = "PRESET")]
    preset_add: Option<String>,

    /// Remove mods from a preset
    #[arg(long, value_name = "PRESET")]
    preset_remove: Option<String>,

    /// List presets
    #[arg(long, short)]
    list_presets: bool,

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

    let beamng_dir = if let Some(dir) = args.custom_data_dir {
        if dir.try_exists()? {
            dir
        } else {
            return Err(beam_mm::Error::DirNotFound { dir });
        }
    } else {
        beamng_dir_default()?
    };

    let beamng_version = beam_mm::game_version(&beamng_dir)?;
    let mods_dir = mods_dir(&beamng_dir, &beamng_version)?;
    let beammm_dir = beammm_dir()?;

    let presets_dir = presets_dir(&beammm_dir)?;

    let mut beamng_mod_cfg = beam_mm::game::ModCfg::load_from_path(&mods_dir)?;

    if args.list_presets {
        for preset in beam_mm::Preset::list(&presets_dir)? {
            println!("{}", preset);
        }
    }
    if let Some(preset) = args.create_preset {
        let preset = beam_mm::Preset::new(preset, args.mods.clone().unwrap_or(vec![]));
        preset.save_to_path(&presets_dir)?;
    }
    if let Some(preset) = args.delete_preset {
        let confirmation = beam_mm::confirm_cli(
            &format!("Are you sure you want to delete preset '{}'?", preset),
            false,
            args.confirm_all,
        )?;
        if confirmation {
            beam_mm::Preset::delete(&preset, &presets_dir)?;
        }
    }
    if let Some(preset) = args.enable_preset {
        let mut preset = beam_mm::Preset::load_from_path(&preset, &presets_dir)?;
        preset.enable();
        preset.save_to_path(&presets_dir)?;
    }
    if let Some(preset) = args.disable_preset {
        let mut preset = beam_mm::Preset::load_from_path(&preset, &presets_dir)?;
        preset.disable(&mut beamng_mod_cfg)?;
        preset.save_to_path(&presets_dir)?;
    }

    // Handle operations that require args.mods to exist.
    if let Some(mods) = args.mods {
        // Check of mods argument is "all"
        let all_mods = Some(String::from("all")) == mods.first().map(|s| s.to_lowercase());

        if args.enable {
            if all_mods {
                let confirmation = beam_mm::confirm_cli(
                    "Are you sure you would like to enable all mods?",
                    true,
                    args.confirm_all,
                )?;
                if confirmation {
                    beamng_mod_cfg.set_all_mods_active(true)?;
                }
            } else {
                beamng_mod_cfg.set_mods_active(&mods, true)?;
            }
        }
        if args.disable {
            if all_mods {
                let confirmation = beam_mm::confirm_cli(
                    "Are you sure you would like to disable all mods?",
                    false,
                    args.confirm_all,
                )?;
                if confirmation {
                    beamng_mod_cfg.set_all_mods_active(false)?;
                }
            } else {
                beamng_mod_cfg.set_mods_active(&mods, false)?;
            }
        }
        if let Some(preset) = args.preset_add {
            let mut preset = beam_mm::Preset::load_from_path(&preset, &presets_dir)?;
            preset.add_mods(&mods);
            preset.save_to_path(&presets_dir)?;
        }
        if let Some(preset) = args.preset_remove {
            let mut preset = beam_mm::Preset::load_from_path(&preset, &presets_dir)?;
            preset.remove_mods(&mods);
            preset.save_to_path(&presets_dir)?;
        }
    }

    beamng_mod_cfg.apply_presets(&presets_dir)?;
    beamng_mod_cfg.save_to_path(&mods_dir)?;

    Ok(())
}
