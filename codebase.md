
# File: Cargo.toml
```toml
[package]
name = "rs-clean"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4.5.53", features = ["derive"] }
walkdir = "2.5.0"

```

# File: test_projects/php_app/composer.json
```json

```

# File: test_projects/rust_app/Cargo.toml
```toml

```

# File: test_projects/dotnet_app/MyProject.csproj
```csproj

```

# File: test_projects/node_app/package.json
```json

```

# File: test_projects/python_app/pyproject.toml
```toml

```

# File: src/main.rs
```rs
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
        Commands::CombineCode {
            path,
            output,
            include,
            exclude,
        } => {
            combine_code(path, output.as_deref(), &include, &exclude);
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

fn combine_code(root: &Path, output_path: Option<&Path>, include: &[String], exclude: &[String]) {
    use std::io::Write;

    if !root.exists() || !root.is_dir() {
        eprintln!("❌ Error: Invalid directory path: {:?}", root);
        return;
    }

    let mut output_writer: Box<dyn Write> = match output_path {
        Some(path) => {
            println!("📝 Combining code from {:?} into {:?}", root, path);
            match fs::File::create(path) {
                Ok(file) => Box::new(file),
                Err(e) => {
                    eprintln!("❌ Error creating output file: {}", e);
                    return;
                }
            }
        }
        None => Box::new(std::io::stdout()),
    };

    let ignored_folders = [
        "node_modules",
        "target",
        "vendor",
        ".git",
        ".svn",
        ".hg",
        ".idea",
        ".vscode",
        "dist",
        "build",
        "coverage",
        "__pycache__",
    ];

    let ignored_files = [
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "Cargo.lock",
        "composer.lock",
        ".DS_Store",
        "Thumbs.db",
        ".env",
    ];

    let walker = WalkDir::new(root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_str().unwrap_or("");

        // Always enter the root directory, even if it starts with '.' (like ".")
        if e.depth() == 0 {
            return true;
        }

        // Skip hidden folders (starting with .) EXCEPT for config files we might want strictly at root
        // But usually hidden folders like .git are ignored.
        // We'll trust the explicit list + . starts.
        if name.starts_with('.') {
            return false;
        }

        if e.file_type().is_dir() {
            return !ignored_folders.contains(&name);
        }

        true
    });

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // 1. Skip ignored files
            if ignored_files.contains(&file_name) || file_name.starts_with('.') {
                continue;
            }

            // Skip the output file itself if it's in the list
            if let Some(out_path) = output_path {
                if let Ok(canon_p) = path.canonicalize() {
                    if let Ok(canon_out) = out_path.canonicalize() {
                        if canon_p == canon_out {
                            continue;
                        }
                    }
                }
            }

            // 2. Check extensions
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                // If specific includes are set, must match one of them
                if !include.is_empty() && !include.contains(&ext.to_string()) {
                    continue;
                }

                // If in exclude list, skip
                if exclude.contains(&ext.to_string()) {
                    continue;
                }

                // Skip binaries / unlikely text files (heuristic)
                let skip_exts = [
                    "png", "jpg", "jpeg", "gif", "ico", "svg", "woff", "woff2", "ttf", "eot",
                    "mp4", "webm", "zip", "tar", "gz", "exe", "dll", "so", "dylib", "class", "pyc",
                ];
                if skip_exts.contains(&ext) {
                    continue;
                }
            } else {
                // No extension? usually skip unless user specifically asked for it via include (handled above)
                // or if include is empty, we might skip to be safe, or include simple text files like LICENSE, Makefile
                let known_text_files = ["Makefile", "Dockerfile", "LICENSE", "README"];
                let is_known = known_text_files.iter().any(|f| file_name.ends_with(f)); // rough check

                if !include.is_empty() && !is_known {
                    continue;
                }
            }

            // 3. Read and Append
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Try to make path relative to root for cleaner headers
                    let rel_path = path.strip_prefix(root).unwrap_or(path);
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

                    let header = format!("\n# File: {}\n```{}\n", rel_path.display(), ext);
                    let footer = "\n```\n";

                    if let Err(e) = output_writer
                        .write_all(header.as_bytes())
                        .and_then(|_| output_writer.write_all(content.as_bytes()))
                        .and_then(|_| output_writer.write_all(footer.as_bytes()))
                    {
                        eprintln!("❌ Error writing to output: {}", e);
                    }
                }
                Err(_) => {
                    // Likely binary or non-utf8, skip silently
                }
            }
        }
    }

    if output_path.is_some() {
        println!("✅ Successfully combined code.");
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

    // Use a mutable walker so we can skip directories we've already handled
    let mut walker = WalkDir::new(root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_str().unwrap_or("");
        // Skip common hidden folders like .git to speed up scan
        !name.starts_with('.') || name == ".venv"
    });

    while let Some(entry) = walker.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading directory: {}", e);
                continue;
            }
        };

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
                            // Important: Delete first, then skip descending
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

                        // optimization: Don't scan inside folders we just deleted or marked for deletion
                        walker.skip_current_dir();
                        break; // Stop checking other rules for this folder
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

```
