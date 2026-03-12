use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
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

#[derive(ValueEnum, Clone, Debug)]
enum SoundPreset {
    Success,
    Error,
    Deploy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    #[serde(default = "default_min_duration_ms")]
    min_duration_ms: u64,
    #[serde(default = "default_deploy_command_prefixes")]
    deploy_command_prefixes: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_duration_ms: default_min_duration_ms(),
            deploy_command_prefixes: default_deploy_command_prefixes(),
        }
    }
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

fn config_path() -> Result<PathBuf, Box<dyn Error>> {
    Ok(sounds_dir()?.join("config.toml"))
}

fn debug_logging_enabled() -> bool {
    matches!(env::var("TERMEME_DEBUG").as_deref(), Ok("1" | "true" | "TRUE"))
}

fn debug_log(message: &str) {
    if debug_logging_enabled() {
        eprintln!("termeme: {message}");
    }
}

fn default_min_duration_ms() -> u64 {
    1500
}

fn default_deploy_command_prefixes() -> Vec<String> {
    vec![
        "git push".to_string(),
        "pnpm deploy".to_string(),
        "npm publish".to_string(),
        "vercel --prod".to_string(),
    ]
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

fn load_config() -> Result<Config, Box<dyn Error>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }

    let raw = fs::read_to_string(&path)?;
    Ok(toml::from_str(&raw)?)
}

fn run_hook(command: &str, exit_code: i32, duration_ms: u64) {
    let config = match load_config() {
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

fn choose_preset(config: &Config, command: &str, exit_code: i32, duration_ms: u64) -> Option<SoundPreset> {
    // Ignore tiny commands so your shell doesn't chirp at every `cd` and `ls`
    if duration_ms < config.min_duration_ms {
        return None;
    }

    let command = command.trim();

    if config
        .deploy_command_prefixes
        .iter()
        .any(|prefix| command.starts_with(prefix))
    {
        return Some(SoundPreset::Deploy);
    }

    if exit_code == 0 {
        Some(SoundPreset::Success)
    } else {
        Some(SoundPreset::Error)
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

    let config_path = config_path()?;
    if config_path.exists() {
        println!("Skipped config.toml, already exists");
    } else {
        let default_config = toml::to_string_pretty(&Config::default())?;
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
    println!("Config file: {}", config_path()?.display());

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
