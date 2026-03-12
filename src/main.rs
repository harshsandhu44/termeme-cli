use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::Path;
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
        path: String,
    },
    Init,
    Doctor,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Play { path } => {
            play_sound(&path)?;
        }
        Commands::Init => {
            println!("Initializing termeme...");
        }
        Commands::Doctor => {
            println!("Checking termsound setup...");

            let status = Command::new("which").arg("afplay").status()?;

            if status.success() {
                println!("afplay is available");
            } else {
                println!("afplay is missing");
            }
        }
    }

    Ok(())
}

fn play_sound(path: &str) -> Result<(), Box<dyn Error>> {
    if !Path::new(path).exists() {
        println!("{}", path);
        return Err(format!("Sound file not found: {}", path).into());
    }

    let status = Command::new("afplay").arg(path).status()?;

    if !status.success() {
        return Err("afplay failed to play the sound".into());
    }

    Ok(())
}

