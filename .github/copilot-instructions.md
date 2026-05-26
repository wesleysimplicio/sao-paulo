# Copilot instructions

Mirrors `AGENTS.md` / `CLAUDE.md`. Keep the three aligned.

## MANDATORY: code through simplicio-cli

Before writing or changing any code in this repository, set up and use
[simplicio-cli](https://github.com/wesleysimplicio/simplicio-cli). This is required
for every coding task.

```bash
bash scripts/setup-simplicio.sh          # install (idempotent)
simplicio index --stack rust             # once per stack
simplicio task "<change>" --target <file> --criteria "<acceptance>" --constraints "- build passes"
simplicio smoke                          # validate provider (needs SIMPLICIO_MODEL + key)
```

Providers: Anthropic native via `ANTHROPIC_API_KEY`; any OpenAI-compatible provider via
`SIMPLICIO_MODEL` + `SIMPLICIO_BASE_URL` + key.

## Repository

`@wesleysimplicio/llm-project-mapper` — Rust engine (`lpm`) + Node CLI. Commands:
`npm test`, `npm run lint`, `cargo build --release`, `cargo test`.

## Rules

- Use simplicio-cli for code tasks; do not hand-edit to bypass it.
- No emojis in source code. English for generated content and identifiers.
