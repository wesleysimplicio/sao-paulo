//! `lpm` — native project mapper CLI.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use lpm::profile::build_profile;
use lpm::render::{render_architecture_map, render_domain_map, render_json};
use lpm::virality::{candidates_from_payload, score_batch, ScoreOptions, ScoringWeights};
use lpm::yool::build_default_space;
use serde_json::{Map, Value};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!(
        r#"lpm (llm-project-mapper, native) v{VERSION}

Map a project locally so an LLM agent has context before it programs.

USAGE
  lpm [map] [path] [options]
  lpm yool [options]
  lpm virality --input <file.json> [--json]
  lpm hamt [project-root] [--source <AGENTS.md>] [--output <.catalog/agents.json>]

OPTIONS (map)
  --json        Print the structured map as JSON to stdout (no files written)
  --write       Write docs/architecture-map.md and docs/domain-map.md (default)
  --dry-run     Inspect and print a summary without writing files

OPTIONS (yool)
  --depth N         Hierarchy depth (default 4)
  --branching N     Children per level (default 32 -> 32^4 = 1,048,576)
  --threshold N     Compression threshold (default 128)
  --json            Print the tuple-space snapshot as JSON

OPTIONS (virality)
  --input <file>    JSON candidate object or array (X For You signals)
  --json            Emit the score report(s) as JSON

OPTIONS (hamt)
  [project-root]    Root used to resolve defaults (default .)
  --source <path>   AGENTS.md to parse (default <root>/AGENTS.md)
  --output <path>   Catalog JSON output (default <root>/.catalog/agents.json)

GLOBAL
  -V, --version Print version
  -h, --help    Print this help

EXAMPLES
  lpm
  lpm map .
  lpm --json
  lpm /path/to/project --dry-run
  lpm yool --depth 4 --branching 32
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

fn run_yool(rest: Vec<String>) -> ExitCode {
    let mut json_out = false;
    let mut depth: i64 = 4;
    let mut branching: i64 = 32;
    let mut threshold: i64 = 128;

    let mut iter = rest.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--json" => json_out = true,
            "--depth" => depth = iter.next().and_then(|v| v.parse().ok()).unwrap_or(depth),
            "--branching" => {
                branching = iter
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(branching)
            }
            "--threshold" => {
                threshold = iter
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(threshold)
            }
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("Unknown yool option: {other}");
                return ExitCode::from(1);
            }
        }
    }

    let (mut space, root) = build_default_space();
    space.spawn_agent(root, "hamt_builder", {
        let mut d = Map::new();
        d.insert("status".into(), Value::String("ready".into()));
        d
    });
    let receipt = match space.batch_spawn(
        root,
        "codex_worker",
        depth,
        branching,
        Some(threshold),
        None,
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("batch_spawn error: {e}");
            return ExitCode::from(1);
        }
    };
    space.hookwall("main_wall", "capability_root", "hook");

    let snap = space.snapshot();
    if json_out {
        println!(
            "{}",
            serde_json::to_string_pretty(&snap).unwrap_or_default()
        );
        return ExitCode::SUCCESS;
    }

    let lanes = snap["lanes"].as_object().map(|m| m.len()).unwrap_or(0);
    println!(
        "[Tuple Space Snapshot] {} tuples, {} lane(s)",
        snap["tuples"], lanes
    );
    println!("[Active Agents/Subagents] {}", snap["active_agents"]);
    println!("[Total Agents/Subagents] {}", snap["total_agents"]);
    println!("[Próximo Yool a executar] codex_worker");
    println!(
        "[Resultado parcial] batch_spawn@{} -> {} virtual subagents (depth={}, branching={}, materialized={})",
        receipt.receipt_id, receipt.virtual_agents, receipt.depth, receipt.branching, snap["active_agents"]
    );
    ExitCode::SUCCESS
}

fn run_virality(rest: Vec<String>) -> ExitCode {
    let mut json_out = false;
    let mut input: Option<String> = None;

    let mut iter = rest.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--json" => json_out = true,
            "--input" => input = iter.next(),
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("Unknown virality option: {other}");
                return ExitCode::from(1);
            }
        }
    }

    let path = match input {
        Some(p) => p,
        None => {
            print_help();
            return ExitCode::SUCCESS;
        }
    };

    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {path}: {e}");
            return ExitCode::from(1);
        }
    };
    let payload: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid JSON in {path}: {e}");
            return ExitCode::from(1);
        }
    };

    let candidates = candidates_from_payload(&payload);
    let reports = score_batch(
        &candidates,
        &ScoringWeights::default(),
        &ScoreOptions::default(),
    );

    if json_out {
        let arr: Vec<Value> = reports.iter().map(|r| r.to_value()).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&Value::Array(arr)).unwrap_or_default()
        );
        return ExitCode::SUCCESS;
    }

    for (i, report) in reports.iter().enumerate() {
        println!("Candidate {}", i + 1);
        println!("{}", report.explain());
        println!();
    }
    ExitCode::SUCCESS
}

fn run_hamt(rest: Vec<String>) -> ExitCode {
    let mut project_root: Option<String> = None;
    let mut source: Option<String> = None;
    let mut output: Option<String> = None;

    let mut iter = rest.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--source" => source = iter.next(),
            "--output" => output = iter.next(),
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                eprintln!("Unknown hamt option: {other}");
                return ExitCode::from(1);
            }
            other => {
                if project_root.is_none() {
                    project_root = Some(other.to_string());
                } else {
                    eprintln!("Unexpected argument: {other}");
                    return ExitCode::from(1);
                }
            }
        }
    }

    let root = Path::new(project_root.as_deref().unwrap_or("."));
    let source_path = source
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("AGENTS.md"));
    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join(".catalog").join("agents.json"));

    if !source_path.exists() {
        eprintln!("[build] AGENTS source not found: {}", source_path.display());
        return ExitCode::from(2);
    }

    match lpm::hamt::run(&source_path, &output_path) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[build] {e}");
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.first().map(|s| s == "yool").unwrap_or(false) {
        return run_yool(raw.into_iter().skip(1).collect());
    }
    if raw.first().map(|s| s == "virality").unwrap_or(false) {
        return run_virality(raw.into_iter().skip(1).collect());
    }
    if raw.first().map(|s| s == "hamt").unwrap_or(false) {
        return run_hamt(raw.into_iter().skip(1).collect());
    }

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
