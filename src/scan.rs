//! Filesystem walking and corpus collection.

use std::fs;
use std::path::{Path, PathBuf};

const TEXT_EXTS: &[&str] = &[
    "md", "txt", "json", "jsonc", "yml", "yaml", "toml", "ts", "tsx", "js", "jsx", "mjs", "cjs",
    "py", "go", "rs", "java", "kt", "php", "rb", "cs", "sh", "ps1", "env", "",
];

const WALK_SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "dist",
    "build",
    "out",
    "coverage",
    ".next",
    ".nuxt",
    "playwright-report",
    "test-results",
    ".turbo",
    ".venv",
    "venv",
    "__pycache__",
    ".idea",
    ".vscode",
    "video",
    ".claude",
    ".codex",
    ".agents",
    ".skills",
    ".specs",
    ".github",
    "docs",
    "scripts",
    "tests",
    "target",
];

/// Read a file as UTF-8, returning an empty string on any error.
pub fn read_safe(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn is_skip_dir(name: &str) -> bool {
    WALK_SKIP_DIRS.contains(&name)
}

fn ext_lower(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default()
}

fn is_text_file(path: &Path) -> bool {
    let ext = ext_lower(path);
    TEXT_EXTS.contains(&ext.as_str())
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = match fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    // Deterministic order so generated maps are stable across runs.
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if file_type.is_dir() {
            if is_skip_dir(&name) {
                continue;
            }
            walk(&path, out);
        } else if file_type.is_file() && is_text_file(&path) {
            out.push(path);
        }
    }
}

/// Collect all text files under `cwd`, skipping vendored/generated directories.
pub fn collect_text_files(cwd: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk(cwd, &mut files);
    files
}

/// Concatenate the first 80 files (first 3000 chars each) into a search corpus.
pub fn collect_corpus(files: &[PathBuf]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for file in files.iter().take(80) {
        let content = read_safe(file);
        let snippet: String = content.chars().take(3000).collect();
        parts.push(snippet);
    }
    parts.join("\n")
}

/// Top-level directories (non-hidden, non-skipped), alphabetically, up to 10.
pub fn collect_top_directories(cwd: &Path) -> Vec<String> {
    let mut dirs: Vec<String> = match fs::read_dir(cwd) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|name| !name.starts_with('.') && !is_skip_dir(name))
            .collect(),
        Err(_) => Vec::new(),
    };
    dirs.sort();
    dirs.truncate(10);
    dirs
}
