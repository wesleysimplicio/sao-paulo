//! Project profile: the structured map produced from a local inspection.

use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::detect;
use crate::scan::{collect_corpus, collect_text_files, collect_top_directories, read_safe};
use crate::text::{humanize_name, safe_title};

#[derive(Debug, Clone, Default)]
pub struct Commands {
    pub dev: String,
    pub build: String,
    pub lint: String,
    pub test: String,
    pub e2e: String,
    pub validate: String,
    pub evidence: String,
    pub install: String,
}

#[derive(Debug, Clone)]
pub struct Feature {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub cwd: String,
    pub today: String,
    pub project_mode: String,
    pub product_title: String,
    pub stack_label: String,
    pub package_manager: String,
    pub commands: Commands,
    pub frontend_url: String,
    pub backend_url: String,
    pub frontend_health: String,
    pub backend_health: String,
    pub database: String,
    pub auth_flow: String,
    pub team: String,
    pub domain: String,
    pub domain_label: String,
    pub entities: Vec<String>,
    pub features: Vec<Feature>,
    pub todos: Vec<String>,
    pub integrations: Vec<String>,
    pub top_dirs: Vec<String>,
    pub system_type: String,
    pub frontend_tech: String,
    pub backend_tech: String,
    pub jobs: String,
    pub business_goal: String,
    pub port: String,
    pub personas: Vec<String>,
}

struct Pkg {
    name: String,
    description: String,
    author_name: Option<String>,
    scripts: HashMap<String, String>,
    raw: String,
}

fn parse_pkg(cwd: &Path) -> Pkg {
    let raw = read_safe(&cwd.join("package.json"));
    let value: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);

    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let description = value
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let author_name = match value.get("author") {
        Some(Value::Object(obj)) => obj.get("name").and_then(Value::as_str).map(String::from),
        _ => None,
    };
    let mut scripts = HashMap::new();
    if let Some(Value::Object(obj)) = value.get("scripts") {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                scripts.insert(k.clone(), s.to_string());
            }
        }
    }

    Pkg {
        name,
        description,
        author_name,
        scripts,
        raw,
    }
}

/// Current UTC date as `YYYY-MM-DD` (no external dependency).
fn today_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant's days->civil algorithm.
fn civil_from_days(z0: i64) -> (i64, u32, u32) {
    let z = z0 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn find_port(frontend: &str, backend: &str) -> String {
    let text = format!("{frontend} {backend}");
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b':' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            let len = j - (i + 1);
            if (2..=5).contains(&len) {
                return text[i + 1..j].to_string();
            }
        }
        i += 1;
    }
    "3000".into()
}

fn health(url: &str) -> String {
    if url == "not-applicable" {
        "not-applicable".into()
    } else {
        format!("{url}/")
    }
}

pub fn default_personas() -> Vec<String> {
    vec!["Developer maintainer".into(), "AI execution agent".into()]
}

/// Inspect `cwd` and build the project profile.
pub fn build_profile(cwd: &Path) -> Profile {
    let pkg = parse_pkg(cwd);
    let readme = read_safe(&cwd.join("README.md"));
    let files = collect_text_files(cwd);
    let corpus = collect_corpus(&files);
    let project_mode = "root".to_string();

    let stack = detect::detect_stack(cwd);
    let package_manager = detect::detect_package_manager(cwd);
    let commands = detect::infer_commands(cwd, &stack, &pkg.scripts, &pkg.raw, &package_manager);
    let (frontend_url, backend_url) = detect::detect_urls(&stack, &project_mode);
    let remote = detect::get_git_remote(cwd);

    let product_basis = if !pkg.name.is_empty() {
        pkg.name.clone()
    } else {
        cwd.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    };
    let product_title = humanize_name(&product_basis);

    let domain = detect::infer_domain(&pkg.name, &pkg.description, &readme);
    let domain_label = humanize_name(&domain);
    let team = detect::infer_team(pkg.author_name.as_deref(), Some(&pkg.name), &remote);

    let search_text = format!("{}\n{}", pkg.raw, corpus).to_lowercase();
    let database = detect::infer_database(&search_text);
    let auth_flow = detect::infer_auth_flow(&search_text);
    let integrations = detect::collect_integrations(&search_text);

    let entities = detect::collect_entities(&files);
    let features = detect::collect_features(&files);
    let todos = detect::collect_todos(&files);
    let top_dirs = collect_top_directories(cwd);
    let system_type = detect::infer_system_type(&stack, &project_mode);

    let frontend_tech = if ["next", "react", "vue", "angular"]
        .iter()
        .any(|w| stack.contains(w))
    {
        stack.clone()
    } else {
        "not detected".into()
    };
    let backend_tech = if ["express", "nestjs", "python", "dotnet", "go", "laravel"]
        .iter()
        .any(|w| stack.contains(w))
    {
        stack.clone()
    } else {
        "not detected".into()
    };

    let jobs = if ["worker", "queue", "cron", "job"]
        .iter()
        .any(|w| corpus.to_lowercase().contains(w))
    {
        "detected in repository text".into()
    } else {
        "not detected".into()
    };

    let business_goal = safe_title(
        &pkg.description,
        &format!("reduce discovery time and make {product_title} easier to operate"),
    );
    let port = find_port(&frontend_url, &backend_url);

    Profile {
        cwd: cwd.display().to_string(),
        today: today_iso(),
        project_mode,
        product_title,
        stack_label: stack,
        package_manager,
        frontend_health: health(&frontend_url),
        backend_health: health(&backend_url),
        commands,
        frontend_url,
        backend_url,
        database,
        auth_flow,
        team,
        domain,
        domain_label,
        entities,
        features,
        todos,
        integrations,
        top_dirs,
        system_type,
        frontend_tech,
        backend_tech,
        jobs,
        business_goal,
        port,
        personas: default_personas(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_date_known_values() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2000-03-01 is 11017 days after epoch.
        assert_eq!(civil_from_days(11017), (2000, 3, 1));
    }

    #[test]
    fn port_extraction() {
        assert_eq!(find_port("not-applicable", "http://localhost:4000"), "4000");
        assert_eq!(find_port("not-applicable", "not-applicable"), "3000");
    }
}
