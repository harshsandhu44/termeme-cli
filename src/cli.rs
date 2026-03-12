use crate::sound::SoundPreset;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "termeme")]
#[command(about = "Play terminal sounds for events and commands")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
