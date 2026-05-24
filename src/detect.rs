//! Detection/inference logic ported from `bin/auto-map.js`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::profile::{Commands, Feature};
use crate::scan::read_safe;
use crate::text::{has_word, humanize_name, slugify};

const ENTITY_WORDS: &[&str] = &[
    "user",
    "account",
    "order",
    "invoice",
    "payment",
    "report",
    "project",
    "workspace",
    "task",
    "sprint",
    "skill",
    "agent",
    "customer",
    "team",
    "session",
    "document",
    "message",
    "job",
    "build",
    "release",
];

const FEATURE_DIRS: &[&str] = &["pages", "routes", "features", "app"];

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

/// `"key"` followed by optional whitespace then `:` — a cheap JSON key probe.
fn has_json_key(text: &str, key: &str) -> bool {
    let needle = format!("\"{key}\"");
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(&needle) {
        let after = from + rel + needle.len();
        if text[after..].trim_start().starts_with(':') {
            return true;
        }
        from = after;
        if from >= text.len() {
            break;
        }
    }
    false
}

fn dir_entry_names(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Detect the primary stack label (root variant of the JS detector).
pub fn detect_stack(cwd: &Path) -> String {
    let pkg = cwd.join("package.json");
    if pkg.exists() {
        let pj = read_safe(&pkg);
        if has_json_key(&pj, "next") {
            return "next-ts".into();
        }
        if has_json_key(&pj, "react") {
            return "react-ts".into();
        }
        if has_json_key(&pj, "vue") {
            return "vue-ts".into();
        }
        if pj.contains("\"@nestjs/core\"") || has_json_key(&pj, "nestjs") {
            return "nestjs".into();
        }
        if has_json_key(&pj, "express") {
            return "node-express".into();
        }
        return "node-ts".into();
    }

    let entries = dir_entry_names(cwd);
    if entries
        .iter()
        .any(|f| f.ends_with(".csproj") || f.ends_with(".sln"))
    {
        return "dotnet".into();
    }
    let pyproject = cwd.join("pyproject.toml");
    let requirements = cwd.join("requirements.txt");
    if pyproject.exists() || requirements.exists() {
        let py = format!("{}{}", read_safe(&pyproject), read_safe(&requirements)).to_lowercase();
        if py.contains("django") {
            return "python-django".into();
        }
        if py.contains("fastapi") {
            return "python-fastapi".into();
        }
        if py.contains("flask") {
            return "python-flask".into();
        }
        return "python".into();
    }
    if cwd.join("go.mod").exists() {
        return "go".into();
    }
    if cwd.join("Cargo.toml").exists() {
        return "rust".into();
    }
    if cwd.join("pubspec.yaml").exists() {
        return "flutter".into();
    }
    if cwd.join("composer.json").exists() {
        return if read_safe(&cwd.join("composer.json")).contains("laravel/framework") {
            "laravel".into()
        } else {
            "php".into()
        };
    }
    if cwd.join("Gemfile").exists() {
        return "ruby".into();
    }
    if cwd.join("mix.exs").exists() {
        return "elixir".into();
    }
    if cwd.join("build.gradle.kts").exists() {
        return "kotlin-gradle".into();
    }
    if cwd.join("build.gradle").exists() {
        return "java-gradle".into();
    }
    if cwd.join("pom.xml").exists() {
        return "java-maven".into();
    }
    "unknown".into()
}

pub fn detect_package_manager(cwd: &Path) -> String {
    if cwd.join("pnpm-lock.yaml").exists() {
        return "pnpm".into();
    }
    if cwd.join("yarn.lock").exists() {
        return "yarn".into();
    }
    if cwd.join("bun.lockb").exists() || cwd.join("bun.lock").exists() {
        return "bun".into();
    }
    "npm".into()
}

pub fn infer_install_command(cwd: &Path, stack: &str, pm: &str) -> String {
    if contains_any(
        stack,
        &[
            "next", "react", "vue", "angular", "nestjs", "express", "node",
        ],
    ) {
        return format!("{pm} install");
    }
    if stack.contains("dotnet") {
        return "dotnet restore".into();
    }
    if stack.contains("python") {
        return if cwd.join("pyproject.toml").exists() {
            "python -m pip install -e .".into()
        } else {
            "pip install -r requirements.txt".into()
        };
    }
    if stack.contains("go") {
        return "go mod download".into();
    }
    if stack.contains("rust") {
        return "cargo build".into();
    }
    "review the project manifest and install its dependencies".into()
}

fn first_nonempty(values: &[String]) -> String {
    for v in values {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    String::new()
}

pub fn infer_commands(
    cwd: &Path,
    stack: &str,
    scripts: &HashMap<String, String>,
    pkg_raw: &str,
    pm: &str,
) -> Commands {
    let has = |name: &str| {
        scripts
            .get(name)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
    };

    let dev = first_nonempty(&[
        if has("dev") {
            format!("{pm} run dev")
        } else {
            String::new()
        },
        if has("start") {
            format!("{pm} run start")
        } else {
            String::new()
        },
        if stack.contains("dotnet") {
            "dotnet run".into()
        } else {
            String::new()
        },
        if stack.contains("python-fastapi") {
            "uvicorn app:app --reload".into()
        } else {
            String::new()
        },
        if stack.contains("python-django") {
            "python manage.py runserver".into()
        } else {
            String::new()
        },
        if stack.contains("go") {
            "go run .".into()
        } else {
            String::new()
        },
        if stack.contains("rust") {
            "cargo run".into()
        } else {
            String::new()
        },
        format!("{pm} run dev"),
    ]);

    let build = first_nonempty(&[
        if has("build") {
            format!("{pm} run build")
        } else {
            String::new()
        },
        if stack.contains("dotnet") {
            "dotnet build".into()
        } else {
            String::new()
        },
        if stack.contains("python") {
            "python -m compileall .".into()
        } else {
            String::new()
        },
        if stack.contains("go") {
            "go build ./...".into()
        } else {
            String::new()
        },
        if stack.contains("rust") {
            "cargo build".into()
        } else {
            String::new()
        },
    ]);

    let lint = first_nonempty(&[
        if has("lint") {
            format!("{pm} run lint")
        } else {
            String::new()
        },
        if stack.contains("python") {
            "ruff check .".into()
        } else {
            String::new()
        },
        if stack.contains("dotnet") {
            "dotnet format --verify-no-changes".into()
        } else {
            String::new()
        },
        if stack.contains("go") {
            "go vet ./...".into()
        } else {
            String::new()
        },
        if stack.contains("rust") {
            "cargo fmt --check".into()
        } else {
            String::new()
        },
    ]);

    let test = first_nonempty(&[
        if has("test") {
            format!("{pm} test")
        } else {
            String::new()
        },
        if stack.contains("dotnet") {
            "dotnet test".into()
        } else {
            String::new()
        },
        if stack.contains("python") {
            "pytest".into()
        } else {
            String::new()
        },
        if stack.contains("go") {
            "go test ./...".into()
        } else {
            String::new()
        },
        if stack.contains("rust") {
            "cargo test".into()
        } else {
            String::new()
        },
    ]);

    let e2e = first_nonempty(&[
        if has("test:e2e") {
            format!("{pm} run test:e2e")
        } else {
            String::new()
        },
        if has("e2e") {
            format!("{pm} run e2e")
        } else {
            String::new()
        },
        if pkg_raw.to_lowercase().contains("playwright") {
            "npx playwright test".into()
        } else {
            String::new()
        },
    ]);

    let joined = [&lint, &test, &build]
        .iter()
        .filter(|c| !c.is_empty())
        .map(|c| c.as_str())
        .collect::<Vec<_>>()
        .join(" && ");
    let validate = if joined.is_empty() {
        first_nonempty(&[
            test.clone(),
            lint.clone(),
            build.clone(),
            "echo \"Add project validation command here\"".into(),
        ])
    } else {
        joined
    };

    let evidence = if e2e.is_empty() {
        "BASE_URL=http://localhost:3000 npx playwright test --project=chromium".into()
    } else {
        e2e.clone()
    };

    Commands {
        dev,
        build,
        lint,
        test,
        e2e,
        validate,
        evidence,
        install: infer_install_command(cwd, stack, pm),
    }
}

/// Returns `(frontend_url, backend_url)`.
pub fn detect_urls(stack: &str, project_mode: &str) -> (String, String) {
    let na = "not-applicable".to_string();
    if project_mode == "monorepo" {
        return (
            "http://localhost:3000".into(),
            "http://localhost:4000".into(),
        );
    }
    if contains_any(stack, &["next", "react", "vue", "angular"]) {
        return ("http://localhost:3000".into(), na);
    }
    if stack.contains("nestjs") {
        return (na, "http://localhost:3000".into());
    }
    if stack.contains("express") {
        return (na, "http://localhost:4000".into());
    }
    if stack.contains("python-fastapi") {
        return (na, "http://localhost:8000".into());
    }
    if stack.contains("python-django") {
        return (
            "http://localhost:8000".into(),
            "http://localhost:8000".into(),
        );
    }
    if stack.contains("dotnet") {
        return (na, "http://localhost:5000".into());
    }
    if stack.contains("go") {
        return (na, "http://localhost:8080".into());
    }
    ("http://localhost:3000".into(), na)
}

pub fn infer_database(text_lower: &str) -> String {
    if text_lower.contains("postgres")
        || has_word(text_lower, "pg")
        || text_lower.contains("npgsql")
    {
        return "PostgreSQL".into();
    }
    if text_lower.contains("mysql") || text_lower.contains("mariadb") {
        return "MySQL/MariaDB".into();
    }
    if text_lower.contains("sqlite") {
        return "SQLite".into();
    }
    if text_lower.contains("mongodb") || text_lower.contains("mongoose") {
        return "MongoDB".into();
    }
    if text_lower.contains("redis") {
        return "Redis".into();
    }
    "none documented".into()
}

pub fn infer_auth_flow(text_lower: &str) -> String {
    if text_lower.contains("next-auth") || text_lower.contains("authjs") {
        return "NextAuth/Auth.js session flow".into();
    }
    if text_lower.contains("clerk") {
        return "Clerk hosted authentication".into();
    }
    if text_lower.contains("auth0") {
        return "Auth0 OAuth/OIDC".into();
    }
    if let Some(i) = text_lower.find("firebase") {
        if text_lower[i..].contains("auth") {
            return "Firebase Authentication".into();
        }
    }
    if text_lower.contains("jwt") || text_lower.contains("bearer token") {
        return "JWT bearer tokens".into();
    }
    if contains_any(text_lower, &["login", "signin", "signup", "password"]) {
        return "application-managed login flow".into();
    }
    "not detected".into()
}

pub fn infer_team(author_name: Option<&str>, pkg_name: Option<&str>, remote: &str) -> String {
    if let Some(name) = author_name {
        if !name.trim().is_empty() {
            return format!("{}-team", slugify(name));
        }
    }
    if let Some(pn) = pkg_name {
        if pn.starts_with('@') && pn.contains('/') {
            let scope = pn[1..].split('/').next().unwrap_or("");
            return format!("{scope}-team");
        }
    }
    if !remote.is_empty() {
        if let Some(owner) = remote_owner(remote) {
            return format!("{}-team", slugify(&owner));
        }
    }
    "core-team".into()
}

fn remote_owner(remote: &str) -> Option<String> {
    let trimmed = remote.trim().trim_end_matches(".git").trim_end_matches('/');
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() < 2 {
        return None;
    }
    let owner_seg = parts[parts.len() - 2];
    let owner = owner_seg.rsplit(':').next().unwrap_or(owner_seg);
    if owner.is_empty() {
        None
    } else {
        Some(owner.to_string())
    }
}

pub fn infer_domain(pkg_name: &str, description: &str, readme: &str) -> String {
    let text = format!("{pkg_name} {description} {readme}").to_lowercase();
    if contains_any(
        &text,
        &[
            "agent",
            "developer",
            "repo",
            "workflow",
            "ci",
            "spec",
            "prompt",
            "codex",
            "claude",
        ],
    ) {
        return "developer-tools".into();
    }
    if contains_any(
        &text,
        &["payment", "billing", "invoice", "checkout", "stripe"],
    ) {
        return "payments".into();
    }
    if contains_any(&text, &["health", "clinic", "patient", "medical"]) {
        return "healthcare".into();
    }
    if contains_any(&text, &["crm", "lead", "sales"]) {
        return "sales-ops".into();
    }
    let humanized = humanize_name(pkg_name).to_lowercase();
    let slug = slugify(&humanized);
    if slug.is_empty() {
        "product-operations".into()
    } else {
        slug
    }
}

pub fn get_git_remote(cwd: &Path) -> String {
    Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

pub fn collect_entities(files: &[PathBuf]) -> Vec<String> {
    let mut scores: HashMap<String, u32> = HashMap::new();
    for file in files {
        let stem = file
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let base: String = stem
            .chars()
            .map(|c| {
                if c == '-' || c == '_' || c == '.' {
                    ' '
                } else {
                    c
                }
            })
            .collect();
        let base = base.trim();
        let base_lower = base.to_lowercase();
        if !ENTITY_WORDS.iter().any(|w| has_word(&base_lower, w)) {
            continue;
        }
        for token in base.split_whitespace() {
            let token_lower = token.to_lowercase();
            if ENTITY_WORDS.iter().any(|w| has_word(&token_lower, w)) {
                *scores.entry(token_lower).or_insert(0) += 1;
            }
        }
    }

    if scores.is_empty() {
        return vec!["Project".into(), "Workflow".into(), "Documentation".into()];
    }

    let mut ranked: Vec<(String, u32)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(6)
        .map(|(name, _)| humanize_name(&name))
        .collect()
}

pub fn collect_features(files: &[PathBuf]) -> Vec<Feature> {
    let mut order: Vec<String> = Vec::new();
    let mut by_name: HashMap<String, String> = HashMap::new();

    for file in files {
        let normalized = file.to_string_lossy().replace('\\', "/");
        let components: Vec<&str> = normalized.split('/').collect();
        let mut found: Option<&str> = None;
        for (i, comp) in components.iter().enumerate() {
            if FEATURE_DIRS.contains(&comp.to_lowercase().as_str()) && i + 1 < components.len() {
                found = Some(components[i + 1]);
                break;
            }
        }
        if let Some(seg) = found {
            let name = humanize_name(seg);
            if !by_name.contains_key(&name) {
                order.push(name.clone());
            }
            by_name.insert(name, normalized);
        }
    }

    if order.is_empty() {
        return vec![Feature {
            name: "Project bootstrap".into(),
            source: "root manifest + starter docs".into(),
        }];
    }

    order
        .into_iter()
        .take(6)
        .map(|name| Feature {
            source: by_name.get(&name).cloned().unwrap_or_default(),
            name,
        })
        .collect()
}

pub fn collect_todos(files: &[PathBuf]) -> Vec<String> {
    let mut todos = Vec::new();
    for file in files {
        let basename = file
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let content = read_safe(file);
        for (idx, line) in content.lines().enumerate() {
            if line.contains("TODO") || line.contains("FIXME") || line.contains("HACK") {
                todos.push(format!("{}:{} \u{2014} {}", basename, idx + 1, line.trim()));
                if todos.len() >= 8 {
                    return todos;
                }
            }
        }
    }
    todos
}

pub fn collect_integrations(text_lower: &str) -> Vec<String> {
    let mut out = Vec::new();
    if text_lower.contains("github") || has_word(text_lower, "gh") {
        out.push("GitHub".to_string());
    }
    if text_lower.contains("stripe") {
        out.push("Stripe".to_string());
    }
    if text_lower.contains("openai") {
        out.push("OpenAI".to_string());
    }
    if text_lower.contains("playwright") {
        out.push("Playwright".to_string());
    }
    if text_lower.contains("auth0") {
        out.push("Auth0".to_string());
    }
    if text_lower.contains("clerk") {
        out.push("Clerk".to_string());
    }
    if text_lower.contains("sentry") {
        out.push("Sentry".to_string());
    }
    if text_lower.contains("redis") {
        out.push("Redis".to_string());
    }
    if text_lower.contains("postgres") || has_word(text_lower, "pg") {
        out.push("PostgreSQL".to_string());
    }
    if out.is_empty() {
        out.push("none detected".to_string());
    }
    out
}

pub fn infer_system_type(stack: &str, project_mode: &str) -> String {
    if project_mode == "monorepo" {
        return "MONOREPO".into();
    }
    if contains_any(stack, &["next", "react", "vue", "angular"]) {
        return "FRONTEND_ONLY".into();
    }
    if contains_any(
        stack,
        &[
            "nestjs", "express", "fastapi", "django", "dotnet", "go", "laravel",
        ],
    ) {
        return "API".into();
    }
    "FULLSTACK".into()
}
