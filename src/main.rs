mod app;
mod cli;
mod config;
mod hook;
mod sound;

use app::run;
use clap::Parser;
use cli::Cli;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    run(Cli::parse())
}
