use clap::{Parser, Subcommand, ValueEnum};
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
            if let Some(preset) = choose_preset(&command, exit_code, duration_ms) {
                let path = preset_path(&preset)?;
                play_sound(&path)?;
            }
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

fn choose_preset(command: &str, exit_code: i32, duration_ms: u64) -> Option<SoundPreset> {
    // Ignore tiny commands so your shell doesn't chirp at every `cd` and `ls`
    if duration_ms < 1500 {
        return None;
    }

    let command = command.trim();

    if command.starts_with("git push")
        || command.starts_with("pnpm deploy")
        || command.starts_with("npm publish")
        || command.starts_with("vercel --prod")
    {
        return Some(SoundPreset::Deploy);
    }

    if exit_code == 0 {
        Some(SoundPreset::Success)
    } else {
        Some(SoundPreset::Error)
    }
}

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

    println!("Sound directory ready: {}", target_dir.display());
    Ok(())
}

fn doctor() -> Result<(), Box<dyn Error>> {
    println!("Checking termeme setup...");

    let afplay_status = Command::new("which").arg("afplay").status()?;
    if afplay_status.success() {
        println!("afplay is available");
    } else {
        println!("afplay is missing");
    }

    let user_dir = sounds_dir()?;
    println!("User sound directory: {}", user_dir.display());

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
