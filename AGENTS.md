# AGENTS.md

Guidance for any coding agent (Claude Code, Codex, Copilot, Cursor, Aider, Hermes, OpenClaw) working in this repository.

> This file is mirrored in `CLAUDE.md` and `.github/copilot-instructions.md`. Keep the three aligned: edit `AGENTS.md` and replicate.

## MANDATORY: code through simplicio-cli

Before writing or changing any code in this repository, you MUST set up and use
[simplicio-cli](https://github.com/wesleysimplicio/simplicio-cli). This is non-negotiable
for every coding task.

1. Ensure the CLI is installed (idempotent):

   ```bash
   bash scripts/setup-simplicio.sh
   ```

2. Drive code changes through the CLI instead of editing files blindly:

   ```bash
   simplicio index --stack rust            # once per stack, or after large changes
   simplicio task "<describe the change>" \
     --target <path/to/file> \
     --criteria "<observable acceptance criteria>" \
     --constraints "<- build passes\n- tests pass>"
   ```

This mandate is enforced by a `PreToolUse` hook (`.claude/hooks/pre-code-simplicio.sh`):
`Edit`/`Write` are blocked until `simplicio` is on `PATH`. In an environment with no
network or provider keys, set `SIMPLICIO_BYPASS=1` to bypass the guard.

### Provider configuration

simplicio-cli works with any OpenAI-compatible provider plus Anthropic native.
It needs `SIMPLICIO_MODEL` and a key. Common setups:

- Anthropic: `ANTHROPIC_API_KEY` (used automatically)
- OpenRouter / OpenAI-like: `SIMPLICIO_MODEL` + `SIMPLICIO_BASE_URL` + key
- DeepSeek / OpenAI / GLM / Ollama: provider-specific credentials

Validate connectivity any time with:

```bash
simplicio smoke
```

## This repository

`@wesleysimplicio/llm-project-mapper` — an AI-friendly project scaffold with a Rust
native engine (`lpm`) and a Node CLI (`llm-project-mapper`).

### Real commands

```bash
npm test                 # node --test
npm run lint             # node scripts/lint.js
npm run test:cli         # node bin/cli.js --help
cargo build --release    # build the native engine (binary: lpm)
cargo test               # rust tests
```

## Rules

- Use simplicio-cli for code tasks; do not hand-edit code to bypass it.
- No emojis in source code.
- Generated content and identifiers in English.
- Run the available tests/lint/build before declaring a task done.
