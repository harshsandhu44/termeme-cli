use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "termeme")]
#[command(about = "Play terminal sounds for events and commands")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Play { sound: String },
    Init,
    Doctor,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Play { sound } => {
            println!("Playing sound preset: {}", sound);
        }
        Commands::Init => {
            println!("Initializing termeme...");
        }
        Commands::Doctor => {
            println!("Checking termeme setup...");
        }
    }
}
