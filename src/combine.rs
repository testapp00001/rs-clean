use bytesize::ByteSize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Simple heuristic: 4 chars ~= 1 token
fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 4
}

pub fn combine_code(
    root: &Path,
    output_path: Option<&Path>,
    include: &[String],
    exclude: &[String],
) {
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

        // Always enter the root directory
        if e.depth() == 0 {
            return true;
        }

        if name.starts_with('.') {
            return false;
        }

        if e.file_type().is_dir() {
            return !ignored_folders.contains(&name);
        }

        true
    });

    let mut total_files = 0;
    let mut total_size = 0;
    let mut total_tokens = 0;

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
                    let size = content.len();
                    let tokens = estimate_tokens(&content);

                    total_files += 1;
                    total_size += size as u64;
                    total_tokens += tokens;

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
        println!("📊 Stats:");
        println!("   Files: {}", total_files);
        println!("   Total Size: {}", ByteSize(total_size).to_string());
        println!("   Est. Tokens: {} (Heuristic: chars/4)", total_tokens);
    }
}
