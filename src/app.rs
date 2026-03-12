use crate::cli::{Cli, Commands};
use crate::config::load_config;
use crate::hook::choose_preset;
use crate::sound::{doctor, init_sounds_dir, play_sound, preset_path, sounds_dir};
use std::error::Error;

pub fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::Play { preset } => {
            let path = preset_path(&preset)?;
            println!("Playing {:?}: {}", preset, path.display());
            play_sound(&path)?;
        }
        Commands::Hook {
            exit_code,
            duration_ms,
            command,
        } => run_hook(&command, exit_code, duration_ms),
        Commands::Init => {
            init_sounds_dir()?;
        }
        Commands::Doctor => {
            doctor()?;
        }
    }

    Ok(())
}

fn debug_logging_enabled() -> bool {
    matches!(std::env::var("TERMEME_DEBUG").as_deref(), Ok("1" | "true" | "TRUE"))
}

fn debug_log(message: &str) {
    if debug_logging_enabled() {
        eprintln!("termeme: {message}");
    }
}

fn run_hook(command: &str, exit_code: i32, duration_ms: u64) {
    let sounds_dir = match sounds_dir() {
        Ok(path) => path,
        Err(error) => {
            debug_log(&format!("failed to resolve sounds dir: {error}"));
            return;
        }
    };

    let config = match load_config(&sounds_dir) {
        Ok(config) => config,
        Err(error) => {
            debug_log(&format!("failed to load config: {error}"));
            return;
        }
    };

    let Some(preset) = choose_preset(&config, command, exit_code, duration_ms) else {
        return;
    };

    let path = match preset_path(&preset) {
        Ok(path) => path,
        Err(error) => {
            debug_log(&format!("failed to resolve preset path: {error}"));
            return;
        }
    };

    if let Err(error) = play_sound(&path) {
        debug_log(&format!("failed to play sound: {error}"));
    }
}
