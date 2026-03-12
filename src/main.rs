mod config;
mod hook;
mod sound;

use clap::{Parser, Subcommand};
use config::load_config;
use hook::choose_preset;
use std::error::Error;
use sound::{doctor, init_sounds_dir, play_sound, preset_path, sounds_dir, SoundPreset};

#[derive(Parser)]
#[command(name = "termeme")]
#[command(about = "Play terminal sounds for events and commands")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Play {
        #[arg(value_enum)]
        preset: SoundPreset,
    },
    Hook {
        #[arg(long)]
        exit_code: i32,
        #[arg(long)]
        duration_ms: u64,
        #[arg(long)]
        command: String,
    },
    Init,
    Doctor,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

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
        } => {
            run_hook(&command, exit_code, duration_ms);
        }
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
