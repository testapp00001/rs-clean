use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// rs-clean: A disk cleanup tool for developers.
#[derive(Parser)]
#[command(name = "rs-clean")]
#[command(version = "0.1.0")]
#[command(about = "Scans and cleans up project dependency folders", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show version information
    Version,
    /// Greet a person
    Greet {
        /// Name of the person to greet
        #[arg(short = 'n', long = "name")]
        name: String,

        /// Number of times to greet
        #[arg(short = 'c', long = "count", default_value_t = 1)]
        count: u8,
    },
    /// Scan and clean up dependency folders (node_modules, target, vendor, etc.)
    Clean {
        /// Root path to start scanning from
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Actually delete the folders (default is dry-run)
        #[arg(short = 'f', long = "force")]
        force: bool,
    },
    /// Combine code files into a single Markdown file
    CombineCode {
        /// Root path to scan
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output file path (default: stdout)
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Comma-separated list of file extensions to include (e.g. rs,py,js)
        #[arg(short = 'i', long = "include", value_delimiter = ',')]
        include: Vec<String>,

        /// Comma-separated list of file extensions to exclude
        #[arg(short = 'e', long = "exclude", value_delimiter = ',')]
        exclude: Vec<String>,
    },
}
