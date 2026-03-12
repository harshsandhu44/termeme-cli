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

fn repo_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn repo_asset_path(filename: &str) -> PathBuf {
    repo_assets_dir().join(filename)
}

fn preset_path(preset: &SoundPreset) -> Result<PathBuf, Box<dyn Error>> {
    let filename = match preset {
        SoundPreset::Success => "success.wav",
        SoundPreset::Error => "error.wav",
        SoundPreset::Deploy => "deploy.wav",
    };

    Ok(sounds_dir()?.join(filename))
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
        let source = repo_asset_path(file);
        let target = target_dir.join(file);

        if !source.exists() {
            eprintln!("Missing repo asset: {}", source.display());
            continue;
        }

        if target.exists() {
            println!("Skipped {}, already exists", file);
        } else {
            fs::copy(&source, &target)?;
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

    let repo_dir = repo_assets_dir();
    println!("Repo assets directory: {}", repo_dir.display());

    let user_dir = sounds_dir()?;
    println!("User sound directory: {}", user_dir.display());

    for file in ["success.wav", "error.wav", "deploy.wav"] {
        let repo_file = repo_dir.join(file);
        let user_file = user_dir.join(file);

        println!(
            "{} -> repo: {}, user: {}",
            file,
            if repo_file.exists() { "found" } else { "missing" },
            if user_file.exists() { "found" } else { "missing" }
        );
    }

    Ok(())
}
