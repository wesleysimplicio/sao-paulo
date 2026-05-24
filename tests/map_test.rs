use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lpm::detect;
use lpm::profile::build_profile;
use lpm::render::{render_architecture_map, render_domain_map, render_json};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("lpm-test-{tag}-{}-{}", std::process::id(), nanos));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &std::path::Path, rel: &str, content: &str) {
    let abs = dir.join(rel);
    if let Some(p) = abs.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(abs, content).unwrap();
}

#[test]
fn express_pg_project_maps_to_api_postgres() {
    let dir = unique_dir("express");
    write(
        &dir,
        "package.json",
        r#"{
  "name": "acme-shop",
  "description": "online store backend",
  "dependencies": { "express": "^4", "pg": "^8" },
  "scripts": { "dev": "node src/index.js", "test": "node --test" }
}"#,
    );
    write(
        &dir,
        "src/index.js",
        "const express = require('express');\n",
    );
    write(&dir, "src/order.js", "// order handling\n");

    let profile = build_profile(&dir);

    assert_eq!(profile.stack_label, "node-express");
    assert_eq!(profile.system_type, "API");
    assert_eq!(profile.database, "PostgreSQL");
    assert_eq!(profile.backend_url, "http://localhost:4000");
    assert_eq!(profile.frontend_url, "not-applicable");
    assert_eq!(profile.product_title, "Acme Shop");
    assert_eq!(profile.commands.dev, "npm run dev");
    assert_eq!(profile.commands.test, "npm test");
    assert!(profile.integrations.contains(&"PostgreSQL".to_string()));
    assert!(profile.entities.contains(&"Order".to_string()));

    let arch = render_architecture_map(&profile);
    assert!(arch.contains("Type: API"));
    assert!(arch.contains("Backend: node-express"));
    assert!(arch.contains("Database: PostgreSQL"));
    assert!(arch.contains("http://localhost:4000"));

    let dom = render_domain_map(&profile);
    assert!(dom.contains("App: Acme Shop"));
    assert!(dom.contains("Developer maintainer"));

    let value = render_json(&profile);
    assert_eq!(value["stack"], "node-express");
    assert_eq!(value["database"], "PostgreSQL");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn python_fastapi_detection() {
    let dir = unique_dir("fastapi");
    write(&dir, "requirements.txt", "fastapi==0.110\nuvicorn\n");
    write(&dir, "app.py", "from fastapi import FastAPI\n");

    let stack = detect::detect_stack(&dir);
    assert_eq!(stack, "python-fastapi");

    let profile = build_profile(&dir);
    assert_eq!(profile.system_type, "API");
    assert_eq!(profile.backend_url, "http://localhost:8000");
    assert_eq!(profile.commands.test, "pytest");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rust_project_detection() {
    let dir = unique_dir("rust");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"thing\"\nversion = \"0.1.0\"\n",
    );
    write(&dir, "src/main.rs", "fn main() {}\n");

    let profile = build_profile(&dir);
    assert_eq!(profile.stack_label, "rust");
    assert_eq!(profile.commands.build, "cargo build");
    assert_eq!(profile.commands.test, "cargo test");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn empty_project_falls_back() {
    let dir = unique_dir("empty");
    write(&dir, "notes.md", "just some notes\n");

    let profile = build_profile(&dir);
    assert_eq!(profile.stack_label, "unknown");
    assert_eq!(profile.system_type, "FULLSTACK");
    assert_eq!(
        profile.entities,
        vec!["Project", "Workflow", "Documentation"]
    );

    let _ = fs::remove_dir_all(&dir);
}
