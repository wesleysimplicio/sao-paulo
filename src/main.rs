//! `lpm` — native project mapper CLI.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use lpm::profile::build_profile;
use lpm::render::{render_architecture_map, render_domain_map, render_json};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!(
        r#"lpm (llm-project-mapper, native) v{VERSION}

Map a project locally so an LLM agent has context before it programs.

USAGE
  lpm [map] [path] [options]

OPTIONS
  --json        Print the structured map as JSON to stdout (no files written)
  --write       Write docs/architecture-map.md and docs/domain-map.md (default)
  --dry-run     Inspect and print a summary without writing files
  -V, --version Print version
  -h, --help    Print this help

EXAMPLES
  lpm
  lpm map .
  lpm --json
  lpm /path/to/project --dry-run
"#
    );
}

/// A file is safe to (over)write only if it is absent or starter-managed.
fn looks_starter_managed(content: &str) -> bool {
    if content.contains("LLM Project Mapper") {
        return true;
    }
    // A `<TOKEN>` placeholder: `<` then 1..=80 non-`>`/non-newline chars then `>`.
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let mut j = i + 1;
            let mut count = 0;
            while j < bytes.len() && bytes[j] != b'>' && bytes[j] != b'\n' && count <= 80 {
                j += 1;
                count += 1;
            }
            if j < bytes.len() && bytes[j] == b'>' && (1..=80).contains(&count) {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn write_if_safe(target: &Path, rel: &str, content: &str) -> bool {
    let abs = target.join(rel);
    let current = fs::read_to_string(&abs).unwrap_or_default();
    if !current.is_empty() && !looks_starter_managed(&current) {
        return false;
    }
    if let Some(parent) = abs.parent() {
        if fs::create_dir_all(parent).is_err() {
            return false;
        }
    }
    fs::write(&abs, content).is_ok()
}

struct Args {
    path: PathBuf,
    json: bool,
    dry_run: bool,
}

fn parse_args() -> Result<Args, i32> {
    let mut path: Option<PathBuf> = None;
    let mut json = false;
    let mut dry_run = false;

    let mut iter = std::env::args().skip(1).peekable();
    // Optional leading `map` subcommand.
    if iter.peek().map(|s| s == "map").unwrap_or(false) {
        iter.next();
    }

    for arg in iter {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Err(0);
            }
            "-V" | "--version" => {
                println!("{VERSION}");
                return Err(0);
            }
            "--json" => json = true,
            "--write" => {}
            "--dry-run" | "--no-write" => dry_run = true,
            other if other.starts_with('-') => {
                eprintln!("Unknown option: {other}");
                print_help();
                return Err(1);
            }
            other => {
                if path.is_none() {
                    path = Some(PathBuf::from(other));
                } else {
                    eprintln!("Unexpected argument: {other}");
                    return Err(1);
                }
            }
        }
    }

    Ok(Args {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        json,
        dry_run,
    })
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(code) => return ExitCode::from(code as u8),
    };

    if !args.path.is_dir() {
        eprintln!("Path is not a directory: {}", args.path.display());
        return ExitCode::from(1);
    }

    let profile = build_profile(&args.path);

    if args.json {
        let value = render_json(&profile);
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_default()
        );
        return ExitCode::SUCCESS;
    }

    let mut written = 0u32;
    if !args.dry_run {
        if write_if_safe(
            &args.path,
            "docs/architecture-map.md",
            &render_architecture_map(&profile),
        ) {
            written += 1;
        }
        if write_if_safe(
            &args.path,
            "docs/domain-map.md",
            &render_domain_map(&profile),
        ) {
            written += 1;
        }
    }

    println!("llm-project-mapper (native) v{VERSION}");
    println!("product:      {}", profile.product_title);
    println!("stack:        {}", profile.stack_label);
    println!("system:       {}", profile.system_type);
    println!("database:     {}", profile.database);
    println!("auth:         {}", profile.auth_flow);
    println!("frontend_url: {}", profile.frontend_url);
    println!("backend_url:  {}", profile.backend_url);
    println!("domain:       {}", profile.domain);
    println!("integrations: {}", profile.integrations.join(", "));
    println!("entities:     {}", profile.entities.join(", "));
    if args.dry_run {
        println!("(dry-run) no files written");
    } else {
        println!(
            "\u{2192} wrote {written} map file(s) under {}/docs",
            profile.cwd
        );
    }

    ExitCode::SUCCESS
}
