use beammm::path::*;
use clap::Parser;
use colored::Colorize;
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

    /// List installed mods
    #[arg(long)]
    list_mods: bool,

    /// List preset mods
    #[arg(long)]
    list_preset_mods: Option<String>,
}

fn main() {
    // Run the main function and call display on errors to get their pretty messages rather than
    // the debug output.
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> beammm::Result<()> {
    let args = Args::parse();

    let beamng_dir = if let Some(dir) = args.custom_data_dir {
        if dir.try_exists()? {
            dir
        } else {
            return Err(beammm::Error::DirNotFound { dir });
        }
    } else {
        beamng_dir_default()?
    };

    let beamng_version = beammm::game_version(&beamng_dir)?;
    let mods_dir = mods_dir(&beamng_dir, &beamng_version)?;
    let beammm_dir = beammm_dir()?;

    let presets_dir = presets_dir(&beammm_dir)?;

    let mut beamng_mod_cfg = beammm::game::ModCfg::load_from_path(&mods_dir)?;

    if let Some(preset_name) = args.list_preset_mods {
        let preset = beammm::Preset::load_from_path(&preset_name, &presets_dir)?;
        let status = if preset.is_enabled() {
            "enabled ".green()
        } else {
            "disabled".red()
        };
        println!("Mods in preset '{}' ({}):", preset_name, status);
        for mod_name in preset.get_mods() {
            println!("{}", mod_name);
        }
    }

    if args.list_presets {
        for preset_name in beammm::Preset::list(&presets_dir)? {
            let preset = beammm::Preset::load_from_path(&preset_name, &presets_dir)?;
            let status = if preset.is_enabled() {
                "enabled ".green()
            } else {
                "disabled".red()
            };
            println!("{} {}", status, preset_name);
        }
    }
    if let Some(preset_name) = args.create_preset {
        // Check if the preset already exists
        if beammm::Preset::exists(&preset_name, &presets_dir) {
            return Err(beammm::Error::PresetExists {
                preset: preset_name,
            });
        }

        let preset = beammm::Preset::new(preset_name.clone(), args.mods.clone().unwrap_or(vec![]));
        preset.save_to_path(&presets_dir)?;
        println!("Preset '{}' created successfully.", preset_name);
        if let Some(_mods) = args.mods.clone() {
            println!("With mods:");
            for mod_name in preset.get_mods() {
                println!("  - {}", mod_name);
            }
        } else {
            println!("No mods added to the preset.");
        }
        println!(
            "Use the --enable-preset and --disable-preset flags to enable or disable the preset."
        );
        println!(
            "Use the --preset-add and --preset-remove flags to add or remove mods from the preset."
        );
    }
    if let Some(preset) = args.delete_preset {
        let confirmation = beammm::confirm_cli(
            &format!("Are you sure you want to delete preset '{}'?", preset),
            false,
            args.confirm_all,
        )?;
        if confirmation {
            match beammm::Preset::delete(&preset, &presets_dir) {
                Ok(_) => (),
                Err(beammm::Error::IO(e)) => match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        println!("Preset '{}' does not exist.", preset);
                        return Ok(());
                    }
                    _ => return Err(beammm::Error::IO(e)),
                },
                Err(e) => {
                    return Err(e);
                }
            }
            println!("Preset '{}' deleted successfully.", preset);
        } else {
            println!("Preset '{}' was not deleted.", preset);
        }
    }
    if let Some(preset_name) = args.enable_preset {
        if preset_name == "all" {
            let confirmation = beammm::confirm_cli(
                "Are you sure you would like to enable all presets?",
                true,
                args.confirm_all,
            )?;
            if confirmation {
                for preset_name in beammm::Preset::list(&presets_dir)? {
                    let mut preset = beammm::Preset::load_from_path(&preset_name, &presets_dir)?;
                    preset.enable();
                    preset.save_to_path(&presets_dir)?;
                    println!("Preset '{}' enabled.", preset_name);
                }
            }
        } else {
            let mut preset = beammm::Preset::load_from_path(&preset_name, &presets_dir)?;
            preset.enable();
            preset.save_to_path(&presets_dir)?;
            println!("Preset '{}' enabled.", preset_name);
        }
    }
    if let Some(preset_name) = args.disable_preset {
        if preset_name == "all" {
            let confirmation = beammm::confirm_cli(
                "Are you sure you would like to disable all presets?",
                false,
                args.confirm_all,
            )?;
            if confirmation {
                for preset_name in beammm::Preset::list(&presets_dir)? {
                    let mut preset = beammm::Preset::load_from_path(&preset_name, &presets_dir)?;
                    preset.disable(&mut beamng_mod_cfg)?;
                    preset.save_to_path(&presets_dir)?;
                    println!("Preset '{}' disabled.", preset_name);
                }
            }
        } else {
            let mut preset = beammm::Preset::load_from_path(&preset_name, &presets_dir)?;
            preset.disable(&mut beamng_mod_cfg)?;
            preset.save_to_path(&presets_dir)?;
            println!("Preset '{}' disabled.", preset_name);
        }
        // let mut preset = beammm::Preset::load_from_path(&preset_name, &presets_dir)?;
        // preset.disable(&mut beamng_mod_cfg)?;
        // preset.save_to_path(&presets_dir)?;
        // println!("Preset '{}' disabled.", preset_name);
    }

    // Handle operations that require args.mods to exist.
    if let Some(mods) = args.mods {
        // Check of mods argument is "all"
        let all_mods = Some(String::from("all")) == mods.first().map(|s| s.to_lowercase());

        if args.enable {
            if all_mods {
                let confirmation = beammm::confirm_cli(
                    "Are you sure you would like to enable all mods?",
                    true,
                    args.confirm_all,
                )?;
                if confirmation {
                    beamng_mod_cfg.set_all_mods_active(true)?;
                    println!("All mods enabled.");
                }
            } else {
                beamng_mod_cfg.set_mods_active(&mods, true)?;
                println!("Mods enabled:");
                for mod_name in mods.iter() {
                    println!("  - {}", mod_name);
                }
            }
        }
        if args.disable {
            if all_mods {
                let confirmation = beammm::confirm_cli(
                    "Are you sure you would like to disable all mods?",
                    false,
                    args.confirm_all,
                )?;
                if confirmation {
                    beamng_mod_cfg.set_all_mods_active(false)?;
                    println!("All mods disabled.");
                }
            } else {
                beamng_mod_cfg.set_mods_active(&mods, false)?;
                println!("Mods disabled:");
                for mod_name in mods.iter() {
                    println!("  - {}", mod_name);
                }
            }
        }
        if let Some(preset_name) = args.preset_add {
            let mut preset = beammm::Preset::load_from_path(&preset_name, &presets_dir)?;
            preset.add_mods(&mods);
            preset.save_to_path(&presets_dir)?;
            println!("Mods added to preset '{}':", preset_name);
        }
        if let Some(preset_name) = args.preset_remove {
            let mut preset = beammm::Preset::load_from_path(&preset_name, &presets_dir)?;
            preset.remove_mods(&mods);
            preset.save_to_path(&presets_dir)?;
            println!("Mods removed from preset '{}':", preset_name);
            for mod_name in mods.iter() {
                println!("  - {}", mod_name);
            }
        }
    }

    if args.list_mods {
        for beamng_mod in beamng_mod_cfg.get_mods() {
            let status = beamng_mod_cfg.is_mod_active(beamng_mod).unwrap(); // Safe to unwrap because we just
                                                                            // got the mods from the config.
            let status_str = if status {
                "enabled ".green()
            } else {
                "disabled".red()
            };

            println!("{} {}", status_str, beamng_mod);
        }
    }

    match beamng_mod_cfg.apply_presets(&presets_dir) {
        Ok(_) => (),
        Err(beammm::Error::PresetsFailed { mods, presets }) => {
            eprintln!("{}", "Failed to apply presets:".red());
            for preset in presets.iter() {
                eprintln!("  - {}", preset);
            }
            eprintln!("Because of the following missing mods:");
            for mod_name in mods {
                eprintln!("  - {}", mod_name);
            }
            eprintln!("{}", "Disabling these presets.".red());
            for preset in presets.iter() {
                let mut preset = beammm::Preset::load_from_path(preset, &presets_dir)?;
                preset.force_disable(&mut beamng_mod_cfg);
                preset.save_to_path(&presets_dir)?;
            }
        }
        Err(e) => return Err(e),
    }
    beamng_mod_cfg.save_to_path(&mods_dir)?;

    Ok(())
}
