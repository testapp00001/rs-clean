mod clean;
mod cli;
mod combine;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Version => {
            println!("rs-clean v0.1.0");
        }
        Commands::Greet { name, count } => {
            for _ in 0..*count {
                println!("Hello {}!", name);
            }
        }
        Commands::Clean { path, force } => {
            clean::clean_projects(path, *force);
        }
        Commands::CombineCode {
            path,
            output,
            include,
            exclude,
        } => {
            combine::combine_code(path, output.as_deref(), include, exclude);
        }
    }
}
