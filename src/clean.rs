use std::fs;
use std::path::{Path};
use walkdir::WalkDir;

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

pub fn clean_projects(root: &Path, force: bool) {
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
