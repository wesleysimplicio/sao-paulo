//! Render the project map artifacts from a `Profile`.

use serde_json::{json, Value};

use crate::profile::Profile;

/// `docs/architecture-map.md` (faithful to the JS renderer).
pub fn render_architecture_map(p: &Profile) -> String {
    let mut service_rows: Vec<String> = Vec::new();
    if p.frontend_url != "not-applicable" {
        service_rows.push(format!(
            "| Frontend | {} | generated default for detected web stack |",
            p.frontend_url
        ));
    }
    if p.backend_url != "not-applicable" {
        service_rows.push(format!(
            "| Backend | {} | generated default for detected server stack |",
            p.backend_url
        ));
    }
    if service_rows.is_empty() {
        service_rows
            .push("| Service | not-applicable | confirm runtime entrypoint manually |".into());
    }

    let key_dirs = if p.top_dirs.is_empty() {
        String::new()
    } else {
        p.top_dirs
            .iter()
            .map(|d| format!("- `{d}` — top-level area detected during bootstrap"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"# Architecture Map

## System Shape

- Type: {system_type}
- Frontend: {frontend_tech}
- Backend: {backend_tech}
- Database: {database}
- Jobs/workers: {jobs}
- External integrations: {integrations}

## Local URLs

| Service | URL | Notes |
|---|---|---|
{service_rows}

## Request Path

```text
Maintainer or AI agent -> project manifest/docs -> runtime entrypoint -> validation commands -> evidence
```

## Key Directories

{key_dirs}

## Authentication

- Flow: {auth_flow}
- Local/demo credentials: not documented
- Token/session storage: not detected
- Common failure mode: missing local environment variables or auth provider configuration

## Observability

- App logs: stdout / terminal output
- API logs: stdout when backend is present
- Job logs: not detected
- Request correlation: not detected

## Deployment

- Environments: local plus CI-managed environments if configured
- CI/CD: GitHub Actions when present under .github/workflows
- Release notes/changelog: CHANGELOG.md
"#,
        system_type = p.system_type,
        frontend_tech = p.frontend_tech,
        backend_tech = p.backend_tech,
        database = p.database,
        jobs = p.jobs,
        integrations = p.integrations.join(", "),
        service_rows = service_rows.join("\n"),
        key_dirs = key_dirs,
        auth_flow = p.auth_flow,
    )
}

/// `docs/domain-map.md` (faithful to the JS renderer).
pub fn render_domain_map(p: &Profile) -> String {
    let concept_rows = p
        .entities
        .iter()
        .take(4)
        .map(|e| {
            format!(
                "| {e} | Conceito recorrente detectado automaticamente no projeto. | .specs/product/DOMAIN.md |"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let entity_rows = p
        .entities
        .iter()
        .map(|e| {
            format!(
                "| {e} | Entidade ou conceito principal identificado no código. | repository structure / docs |"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# Domain Map

## Product Context

- App: {product_title}
- Main users: {users}
- Main business goal: {business_goal}

## Core Concepts

| Concept | Meaning | Source of truth |
|---|---|---|
{concept_rows}

## Critical Rules

| Rule | Expected behavior | Where implemented | How to test |
|---|---|---|---|
| Commands stay documented | Desenvolvimento, validação e evidência precisam estar explícitos | docs/local-setup.md + AGENTS.md | executar os comandos listados |
| Mapping stays current | Mudanças relevantes atualizam docs no mesmo PR | .specs/ + docs/ | revisão de diff |

## Main Entities

| Entity | Description | Storage |
|---|---|---|
{entity_rows}

## Important Flows

### Project bootstrap and validation

1. User/system action: apply the starter and inspect the project.
2. Entry point: repository manifest and local scripts.
3. Main modules: package manifest, docs, tests, validation scripts.
4. Output: actionable project map plus runnable commands.
5. Evidence: lint/test output and Playwright report when available.

## Edge Cases

- Commands absent from the manifest: bootstrap falls back to generic runtime defaults.
- Pre-existing docs owned by the host project: automatic mapping preserves them instead of overwriting.
"#,
        product_title = p.product_title,
        users = p.personas.join(", "),
        business_goal = p.business_goal,
        concept_rows = concept_rows,
        entity_rows = entity_rows,
    )
}

/// Machine-readable project map (the native engine's structured output).
pub fn render_json(p: &Profile) -> Value {
    json!({
        "product_title": p.product_title,
        "stack": p.stack_label,
        "package_manager": p.package_manager,
        "system_type": p.system_type,
        "project_mode": p.project_mode,
        "domain": p.domain,
        "domain_label": p.domain_label,
        "team": p.team,
        "business_goal": p.business_goal,
        "database": p.database,
        "auth_flow": p.auth_flow,
        "frontend_url": p.frontend_url,
        "backend_url": p.backend_url,
        "frontend_health": p.frontend_health,
        "backend_health": p.backend_health,
        "port": p.port,
        "frontend_tech": p.frontend_tech,
        "backend_tech": p.backend_tech,
        "jobs": p.jobs,
        "today": p.today,
        "commands": {
            "dev": p.commands.dev,
            "build": p.commands.build,
            "lint": p.commands.lint,
            "test": p.commands.test,
            "e2e": p.commands.e2e,
            "validate": p.commands.validate,
            "evidence": p.commands.evidence,
            "install": p.commands.install,
        },
        "entities": p.entities,
        "features": p.features.iter().map(|f| json!({"name": f.name, "source": f.source})).collect::<Vec<_>>(),
        "todos": p.todos,
        "integrations": p.integrations,
        "top_dirs": p.top_dirs,
        "personas": p.personas,
    })
}
