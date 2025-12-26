use bytesize::ByteSize;
use ignore::{WalkBuilder, WalkState};
use rayon::prelude::*;
use std::fs;
use std::path::{Path};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

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

/// Calculate directory size using Rayon for parallelism
fn calculate_size(path: &Path) -> u64 {
    WalkBuilder::new(path)
        .build()
        .par_bridge()
        .filter_map(|e| e.ok())
        .map(|e| {
            if e.path().is_file() {
                e.metadata().map(|m| m.len()).unwrap_or(0)
            } else {
                0
            }
        })
        .sum()
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
    } else {
        println!("⚠️  DELETING MODE: Folders will be permanently removed.\n");
    }

    let total_freed = Arc::new(AtomicU64::new(0));
    let found_any = Arc::new(AtomicU64::new(0));

    // Parallel walker to check matches
    WalkBuilder::new(root)
        .threads(num_cpus::get())
        .build_parallel()
        .run(|| {
            let total_freed = total_freed.clone();
            let found_any = found_any.clone();
            Box::new(move |entry| {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return WalkState::Continue,
                };

                let path = entry.path();
                if path.is_dir() {
                    let folder_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    for rule in CLEAN_RULES {
                        if folder_name == rule.folder_name {
                            let parent = path.parent().unwrap_or_else(|| Path::new("."));
                            // We need to check if indicator exists.
                            // Since we are inside a parallel walker, simple exists() check is fine,
                            // but we should avoid expensive ops if possible.
                            // matches_indicator is reasonably fast (stat check).
                            let should_clean = match rule.project_indicator {
                                Some(ind) => matches_indicator(parent, ind),
                                None => true,
                            };

                            if should_clean {
                                found_any.fetch_add(1, Ordering::Relaxed);

                                // Calculate size before deleting (or just for reporting)
                                let size = calculate_size(path);
                                let size_str = ByteSize(size).to_string();

                                if force {
                                    // print! macro might interleave lines in parallel.
                                    // For a CLI tool, usually line buffering handles it okay, but let's see.
                                    println!(
                                        "🗑️  Deleting {:?} ({}) - freeing {}...",
                                        path, rule.description, size_str
                                    );
                                    match fs::remove_dir_all(path) {
                                        Ok(_) => {
                                            total_freed.fetch_add(size, Ordering::Relaxed);
                                        }
                                        Err(e) => println!("   FAILED to delete {:?}: {}", path, e),
                                    }
                                } else {
                                    println!(
                                        "[MATCH] Found {:<12} at {:?} ({}) - size: {}",
                                        rule.folder_name, path, rule.description, size_str
                                    );
                                    total_freed.fetch_add(size, Ordering::Relaxed);
                                }

                                return WalkState::Skip; // Don't scan inside the folder we just cleaned/found
                            }
                        }
                    }
                }
                WalkState::Continue
            })
        });

    let count = found_any.load(Ordering::Relaxed);
    let bytes = total_freed.load(Ordering::Relaxed);

    if count == 0 {
        println!("✨ Everything looks clean!");
    } else {
        if force {
            println!("\n✅ Process complete.");
            println!("🎉 Reclaimed space: {}", ByteSize(bytes).to_string());
        } else {
            println!(
                "\n💡 Potential space to reclaim: {}",
                ByteSize(bytes).to_string()
            );
        }
    }
}
