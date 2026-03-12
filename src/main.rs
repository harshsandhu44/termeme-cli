mod config;
mod hook;

use clap::{Parser, Subcommand, ValueEnum};
use config::{config_path, load_config, serialize_default_config};
use hook::choose_preset;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum SoundPreset {
    Success,
    Error,
    Deploy,
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

fn sounds_dir() -> Result<PathBuf, Box<dyn Error>> {
    let home = env::var("HOME")?;
    Ok(PathBuf::from(home).join(".termeme"))
}

fn debug_logging_enabled() -> bool {
    matches!(env::var("TERMEME_DEBUG").as_deref(), Ok("1" | "true" | "TRUE"))
}

fn debug_log(message: &str) {
    if debug_logging_enabled() {
        eprintln!("termeme: {message}");
    }
}

fn preset_filename(preset: &SoundPreset) -> &'static str {
    match preset {
        SoundPreset::Success => "success.wav",
        SoundPreset::Error => "error.wav",
        SoundPreset::Deploy => "deploy.wav",
    }
}

fn bundled_asset_bytes(filename: &str) -> Option<&'static [u8]> {
    match filename {
        "success.wav" => Some(include_bytes!("../assets/success.wav")),
        "error.wav" => Some(include_bytes!("../assets/error.wav")),
        "deploy.wav" => Some(include_bytes!("../assets/deploy.wav")),
        _ => None,
    }
}

fn preset_path(preset: &SoundPreset) -> Result<PathBuf, Box<dyn Error>> {
    Ok(sounds_dir()?.join(preset_filename(preset)))
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

fn init_sounds_dir() -> Result<(), Box<dyn Error>> {
    let target_dir = sounds_dir()?;
    fs::create_dir_all(&target_dir)?;

    for file in ["success.wav", "error.wav", "deploy.wav"] {
        let target = target_dir.join(file);
        let Some(bytes) = bundled_asset_bytes(file) else {
            eprintln!("Missing bundled asset: {}", file);
            continue;
        };

        if target.exists() {
            println!("Skipped {}, already exists", file);
        } else {
            fs::write(&target, bytes)?;
            println!("Copied {}", file);
        }
    }

    let config_path = config_path(&target_dir);
    if config_path.exists() {
        println!("Skipped config.toml, already exists");
    } else {
        let default_config = serialize_default_config()?;
        fs::write(&config_path, default_config)?;
        println!("Created config.toml");
    }

    println!("Sound directory ready: {}", target_dir.display());
    Ok(())
}

fn command_in_path(command: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&paths).any(|dir| dir.join(command).is_file())
}

fn doctor() -> Result<(), Box<dyn Error>> {
    println!("Checking termeme setup...");

    #[cfg(target_os = "macos")]
    println!(
        "Playback backend: afplay ({})",
        if command_in_path("afplay") {
            "available"
        } else {
            "missing"
        }
    );

    #[cfg(not(target_os = "macos"))]
    println!("Playback backend: unsupported on this platform");

    let user_dir = sounds_dir()?;
    println!("User sound directory: {}", user_dir.display());
    println!("Config file: {}", config_path(&user_dir).display());

    for file in ["success.wav", "error.wav", "deploy.wav"] {
        let user_file = user_dir.join(file);

        println!(
            "{} -> bundled: {}, user: {}",
            file,
            if bundled_asset_bytes(file).is_some() {
                "found"
            } else {
                "missing"
            },
            if user_file.exists() { "found" } else { "missing" }
        );
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn play_sound(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        return Err(format!("Sound file not found: {}", path.display()).into());
    }

    let status = Command::new("afplay").arg(path).status()?;

    if !status.success() {
        return Err("afplay failed to play the sound".into());
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn play_sound(path: &Path) -> Result<(), Box<dyn Error>> {
    let _ = path;
    Err("sound playback is currently only supported on macOS".into())
}
