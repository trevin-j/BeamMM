use clap::Parser;

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
}

fn main() {
    let args = Args::parse();

    if let Some(preset) = args.create_preset {
        crate::create_preset(preset, args.mods.unwrap_or(vec![]));
    }
    if let Some(preset) = args.delete_preset {
        let confirmation = crate::confirm(
            format!("Are you sure you want to delete preset '{}'?", preset),
            false,
            args.confirm_all,
        );
        if confirmation {
            crate::delete_preset(preset);
        }
    }
    if let Some(preset) = args.enable_preset {
        crate::enable_preset(preset);
    }
    if let Some(preset) = args.disable_preset {
        crate::disable_preset(preset);
    }

    // Handle operations that require args.mods to exist.
    if let Some(mods) = args.mods {
        // Check of mods argument is "all"
        let all_mods = Some(String::from("all")) == mods.get(0).map(|s| s.to_lowercase());

        if args.add {
            crate::add_mods(mods)?;
        }
        if args.remove {
            if all_mods {
                let confirmation = crate::confirm(
                    "Are you sure you would like to remove all mods?".into(),
                    false,
                    args.confirm_all,
                );
                if confirmation {
                    crate::remove_all_mods()?;
                }
            } else {
                crate::remove_mods(mods)?;
            }
        }
        if args.update {
            if all_mods {
                crate::update_all_mods()?;
            } else {
                crate::update_mods(mods)?;
            }
        }
        if args.enable {
            if all_mods {
                let confirmation = crate::confirm(
                    "Are you sure you would like to enable all mods?".into(),
                    true,
                    args.confirm_all,
                );
                if confirmation {
                    crate::enable_all_mods()?;
                }
            } else {
                crate::enable_mods(mods);
            }
        }
        if args.disable {
            if all_mods {
                let confirmation = crate::confirm(
                    "Are you sure you would like to disable all mods?".into(),
                    false,
                    args.confirm_all,
                );
                if confirmation {
                    crate::disable_all_mods()?;
                }
            } else {
                crate::disable_mods(mods);
            }
        }
        if let Some(preset) = args.preset_add {
            crate::add_to_preset(preset, mods);
        }
        if let Some(preset) = args.preset_remove {
            crate::remove_from_preset(preset, mods);
        }
    }
}
