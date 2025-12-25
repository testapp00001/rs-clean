use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// rs-clean: A disk cleanup tool for developers.
#[derive(Parser)]
#[command(name = "rs-clean")]
#[command(version = "0.1.0")]
#[command(about = "Scans and cleans up project dependency folders", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
}

struct CleanRule {
    folder_name: &'static str,
    project_indicator: Option<&'static str>,
    description: &'static str,
}

const CLEAN_RULES: &[CleanRule] = &[
    CleanRule {
        folder_name: "node_modules",
        project_indicator: Some("package.json"),
        description: "Node.js dependencies",
    },
    CleanRule {
        folder_name: "target",
        project_indicator: Some("Cargo.toml"),
        description: "Rust build artifacts",
    },
    CleanRule {
        folder_name: "vendor",
        project_indicator: Some("composer.json"),
        description: "PHP dependencies",
    },
    CleanRule {
        folder_name: "venv",
        project_indicator: None,
        description: "Python virtual environment",
    },
    CleanRule {
        folder_name: ".venv",
        project_indicator: None,
        description: "Python virtual environment",
    },
    CleanRule {
        folder_name: "bin",
        project_indicator: Some("*.csproj"),
        description: ".NET build output",
    },
    CleanRule {
        folder_name: "obj",
        project_indicator: Some("*.csproj"),
        description: ".NET intermediate output",
    },
];

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
            clean_projects(path, *force);
        }
    }
}

fn matches_indicator(parent: &Path, indicator: &str) -> bool {
    if indicator.contains('*') {
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                if let Some(ext) = indicator.strip_prefix("*.") {
                    if entry.path().extension().map_or(false, |e| e == ext) {
                        return true;
                    }
                }
            }
        }
        false
    } else {
        parent.join(indicator).exists()
    }
}

fn clean_projects(root: &Path, force: bool) {
    if !root.exists() {
        eprintln!("❌ Error: Path {:?} does not exist.", root);
        eprintln!(
            "Hint: If you are on Windows, ensure you use forward slashes (/) or quote the path if it contains backslashes (\\)."
        );
        return;
    }

    if !root.is_dir() {
        eprintln!("❌ Error: {:?} is not a directory.", root);
        return;
    }

    println!("🔍 Scanning path: {:?}", root);
    if !force {
        println!("⚠️  DRY RUN: No folders will be deleted. Use --force to delete.\n");
    }

    let mut found_any = false;
    let mut total_cleaned = 0;

    let walker = WalkDir::new(root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_str().unwrap_or("");
        // Skip common hidden folders like .git to speed up scan
        !name.starts_with('.') || name == ".venv"
    });

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir() {
            let folder_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            for rule in CLEAN_RULES {
                if folder_name == rule.folder_name {
                    let parent = path.parent().unwrap_or_else(|| Path::new("."));

                    let should_clean = match rule.project_indicator {
                        Some(ind) => matches_indicator(parent, ind),
                        None => true,
                    };

                    if should_clean {
                        found_any = true;
                        if force {
                            print!("🗑️  Deleting {:?} ... ", path);
                            match fs::remove_dir_all(path) {
                                Ok(_) => {
                                    println!("DONE");
                                    total_cleaned += 1;
                                }
                                Err(e) => println!("FAILED: {}", e),
                            }
                        } else {
                            println!(
                                "[MATCH] Found {:<12} at {:?} ({})",
                                rule.folder_name, path, rule.description
                            );
                        }
                    }
                }
            }
        }
    }

    if !found_any {
        println!("✨ Everything looks clean!");
    } else if force {
        println!("\n✅ Successfully cleaned {} folders.", total_cleaned);
    }
}
