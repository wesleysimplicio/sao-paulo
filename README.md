# São Paulo - Brazilian LLM

> 🇧🇷 Brazilian LLM project. 🇺🇸 English — leia em português: [README.pt-BR.md](README.pt-BR.md).
> Live docs site: [wesleysimplicio.github.io/llm-project-mapper](https://wesleysimplicio.github.io/llm-project-mapper/)

**São Paulo** is a Brazilian LLM initiative built on an AI-friendly, stack-neutral toolchain. At its core is a **native Rust project-mapping engine** plus a repository scaffold: drop it into **any** project — new or existing — and any agent CLI (Claude Code, Codex, Cursor, GitHub Copilot, Aider with Deepseek/Kimi/MiniMax/GLM, Hermes, OpenClaw) gets the context it needs to ship work the same day.

> Native speed first. The mapping engine is a single ~560 KB Rust binary that maps a project in milliseconds — see the [benchmark](#benchmark).

![São Paulo - Brazilian LLM hero](assets/llm-project-mapper-hero.png)

> Visual summary: drop the starter into a messy software project and it turns scattered context into structure, reusable skills, tests, docs, and guardrails for AI coding agents.

### Watch: why São Paulo? (53s)

[![Watch the video](video/assets/why-cover.png)](https://github.com/wesleysimplicio/llm-project-mapper/raw/main/video/assets/why-llm-project-mapper.mp4)

> Click the cover to play. Direct link: [`video/assets/why-llm-project-mapper.mp4`](video/assets/why-llm-project-mapper.mp4) · English version: [`video/assets/why-llm-project-mapper-en.mp4`](video/assets/why-llm-project-mapper-en.mp4) · both renders now ship with narration plus burned-in captions.

---

## Benchmark

São Paulo's mapping engine was rewritten in **Rust** (`lpm`) for speed. It is a faithful port of the original Node mapper (`bin/auto-map.js`) and produces **byte-for-byte identical** `docs/architecture-map.md` and `docs/domain-map.md` — but with no Node startup cost.

| Metric | Node (`bin/auto-map.js`) | Native Rust (`lpm`) | Speedup |
|---|---|---|---|
| Single map (sample project) | ~66 ms | ~4 ms | **~16x** |
| 30 maps (amortized) | 1.90 s (~63 ms/run) | 0.12 s (~3.9 ms/run) | **~16x** |
| Map this repository (full tree) | — | ~8 ms | — |
| Binary size | — (needs Node.js runtime) | ~560 KB static binary | — |
| Runtime dependencies | Node.js | none (1 build dep: `serde_json`) | — |

> **Parity & quality gates:** output verified byte-for-byte against the Node mapper, plus `cargo test` (6 unit + 4 integration), `cargo clippy -D warnings`, and `cargo fmt --check` all green.
> Measured on Linux x86-64, `--release` build (`opt-level=3`, LTO). Absolute numbers vary by hardware and repository size; the ~16x ratio comes mostly from eliminating Node process startup and a faster file walk.

Run it yourself:

```bash
cargo build --release
time ./target/release/lpm . --dry-run
```

---

## Operational Docs For Agents

This starter now includes generic, fill-in templates that make any project easier for agents to operate:

- `docs/local-setup.md`: how to install, start, validate and access the project.
- `docs/domain-map.md`: business concepts, critical rules and edge cases.
- `docs/architecture-map.md`: system shape, request path and integrations.
- `docs/features/README.md`: feature documentation template with files, endpoints, rules and evidence.
- `docs/evidence/README.md`: screenshot/video/trace policy and artifact naming.
- `docs/troubleshooting.md`: repeatable diagnosis and fixes.
- `scripts/`: stack-neutral placeholders for start, test and evidence commands.
- `tests/e2e/smoke.spec.ts`: generic Playwright smoke test driven by `BASE_URL`.

Fill these files after installing the starter in a real project. The goal is to reduce discovery time for humans and agents without forcing a framework.

---

## Native engine (Rust) — the São Paulo mapper

São Paulo's project-mapping pass runs as a native, dependency-light Rust binary (`lpm`) for maximum speed (see the [benchmark](#benchmark)). It is a faithful port of the JavaScript mapper in `bin/auto-map.js` — given the same project, it produces byte-identical `docs/architecture-map.md` and `docs/domain-map.md` — but runs as a single static binary with no Node startup cost.

```bash
# Build
cargo build --release        # produces ./target/release/lpm

# Map the current project (writes docs/architecture-map.md + docs/domain-map.md)
./target/release/lpm

# Map a specific path
./target/release/lpm map /path/to/project

# Structured map as JSON (no files written), or inspect without writing
./target/release/lpm --json
./target/release/lpm /path/to/project --dry-run
```

It infers stack, package manager, run/validate/evidence commands, service URLs, domain, entities, features, TODOs, integrations and directory shape from a local inspection — the context an agent needs before it starts programming.

**Why Rust here:** the mapper is the part worth accelerating. See the [Benchmark](#benchmark) section for the numbers (~16x faster per invocation, ~560 KB binary, no runtime dependency). The Node CLI (`npx @wesleysimplicio/llm-project-mapper`) remains the zero-install scaffolding entry point.

Tests and benchmarks:

```bash
cargo test            # unit + integration (detection/parity) tests
cargo clippy          # lint
cargo fmt --check     # format check
```

---

## YOOL tuple-space runtime (`lpm yool`)

São Paulo also ships a native Rust port of the **YOOL / tuple / HAMT** capability-addressing kernel from [`simplicio-prompt`](https://github.com/wesleysimplicio/simplicio-prompt). It implements the deterministic core: Linda-style tuple-space primitives (`out`/`in`/`rd`), Hilbert-indexed tuple maps, content-addressable receipts, env-driven lane concurrency policy, `compress_token`/`prune_idle`, and the headline `batch_spawn` — a lazy hierarchical fan-out that represents **1,000,000+ virtual subagents without enumerating them**.

```bash
lpm yool                              # depth=4, branching=32 -> 1,048,576 virtual subagents
lpm yool --depth 4 --branching 32 --json
```

```text
[Tuple Space Snapshot] 3 tuples, 1 lane(s)
[Active Agents/Subagents] 2
[Total Agents/Subagents] 1048578
[Próximo Yool a executar] codex_worker
[Resultado parcial] batch_spawn@... -> 1048576 virtual subagents (depth=4, branching=32, materialized=2)
```

Only **2** tuples are materialized for a million-subagent task; the rest stay virtual (counted, never enumerated). The snapshot counts match the Python reference kernel exactly. The provider-facing runtime (async lane worker pool, circuit breaker, jittered backoff) stays in the canonical spec; this port covers the deterministic logic. Receipts use a dependency-free FNV-1a content hash (stable content addresses, not the Python blake2b digest).

---

## X virality scoring (`lpm virality`)

São Paulo also knows how to score posts for the X (For You) algorithm — a native Rust port of the [`x-virality-skills`](https://github.com/wesleysimplicio/x-virality-skills) scorer (`src/x_virality_tools/score_simulator.py`). It combines Phoenix per-candidate engagement predictions with the algorithm's scoring weights (reply `13.5`, follow-author `24.0`, profile-click `12.0`, report `-100.0`, …), then applies the negative-score offset, author-diversity decay, and the out-of-network factor.

```bash
lpm virality --input candidates.json          # human-readable report
lpm virality --input candidates.json --json    # JSON score reports
```

`candidates.json` is a single candidate object or an array, each shaped like:

```json
{ "scores": { "favorite": 0.4, "reply": 0.05, "vqv": 0.3 },
  "in_network": true, "video_duration_ms": 8000, "author_id": 1 }
```

The pipeline is `combined_weighted_score → offset → author-diversity → out-of-network`, with VQV (video quality view) signals gated on a minimum video duration. **Output matches the Python `score_simulator` exactly** (combined/offset/diversity/final within 1e-9), verified by parity tests; covered by `cargo test`.

---

## SkillOpt — optimize agent skills like you train a network (`lpm skillopt`)

São Paulo ports the optimization loop of [**SkillOpt**](https://microsoft.github.io/SkillOpt/) (Microsoft Research). SkillOpt *"treats a compact natural-language skill document as the trainable state of a frozen language agent"*: a separate optimizer turns scored rollouts into **bounded add/delete/replace edits** on a single skill document, and an edit is accepted **only when it strictly improves a held-out validation score**. `lpm skillopt` reproduces the published machinery deterministically — a textual learning-rate budget, success/failure minibatch separation, a rejected-edit buffer, and an epoch-wise slow/meta update — and emits the single deployable markdown skill plus a per-step training history.

```bash
lpm skillopt --input rollouts.json --output skill.md   # human-readable summary + optimized skill
lpm skillopt --input rollouts.json --json               # full training history + result as JSON
lpm skillopt --input rollouts.json --epochs 5 --edit-budget 2 --batch-size 4
```

`rollouts.json` carries the starting skill, the scored rollouts (`tasks`, split into `train`/`val`), and a procedure `catalog`:

```json
{
  "skill": { "title": "Code agent", "lessons": [] },
  "catalog": { "abs-paths": "Use absolute paths when editing files." },
  "tasks": [
    { "split": "train", "success": false, "requires": ["abs-paths"] },
    { "split": "val",   "success": false, "requires": ["abs-paths"] }
  ]
}
```

The loop is `rollout (offline) → reflect (success/failure minibatches) → bounded edit → validation gate`, with the edit budget acting as a textual learning rate, a rejected-edit buffer to avoid repeating harmful directions, and an epoch-wise slow update that prunes stale procedures without regressing validation. The control plane is native; the LLM optimizer's reflect step is modeled deterministically (recurrence over offline rollouts) so the result is reproducible and covered by `cargo test`. A worked example ships at [`examples/skillopt-rollouts.json`](examples/skillopt-rollouts.json).

---

## Patterns

- Canonical spec: [YOOL_TUPLE_HAMT.md](YOOL_TUPLE_HAMT.md)
- Receipts schema and storage conventions: [Receipt schema](YOOL_TUPLE_HAMT.md#184-receipt-schema-reference)

The yool / tuple / HAMT pattern is the capability-addressing model this scaffold is standardizing for multi-agent repos. Keep the root spec vendored so agents can reach it from the repository root in one click.

---

## TL;DR — get going in 60 seconds

Pick **one** of the install paths below and run it inside your project folder. The bootstrap now starts an automatic local mapping pass immediately; `INIT.md` becomes an optional refinement step for a stronger agent.

| OS | Recommended one-liner |
|---|---|
| **macOS** | `npx @wesleysimplicio/llm-project-mapper` |
| **Linux** | `npx @wesleysimplicio/llm-project-mapper` |
| **Windows (PowerShell)** | `npx @wesleysimplicio/llm-project-mapper` |
| **Windows (cmd.exe)** | `npx @wesleysimplicio/llm-project-mapper` |

Same command everywhere. No bash dependency, no clone, no global install.

---

## What São Paulo Changes

The point of São Paulo is not “more files”. It is faster agent execution with less ambiguity, less tribal knowledge, and safer delivery loops — starting from a project map produced in milliseconds.

#### 01 · From project chaos to operational structure

![Project transformation](assets/llm-project-mapper-transformation.png)

> Drop the starter into an existing codebase and it converts scattered context into repeatable docs, validation, agent instructions, and delivery guardrails.

#### 02 · Shared context for parallel agents

![Multi-agent collaboration](assets/llm-project-mapper-multi-agent.png)

> Agents stop working as isolated chat sessions and start collaborating around the same project map: architecture, tasks, checks, and output expectations.

#### 03 · A stable foundation for safe speed

![Operational foundation](assets/llm-project-mapper-foundation.png)

> The end state is an agent-ready project foundation: domain context, architecture, workflow, quality gates, and evidence paths that make automation reliable instead of risky.

---

## Prerequisites

| Requirement | macOS | Linux | Windows |
|---|---|---|---|
| **Node.js >= 16.7** (for `npx`) | `brew install node` | `sudo apt install nodejs npm` (Debian/Ubuntu) · `sudo dnf install nodejs npm` (Fedora) · or [nvm](https://github.com/nvm-sh/nvm) | [nodejs.org installer](https://nodejs.org) or `winget install OpenJS.NodeJS.LTS` |
| **Git** | preinstalled / `brew install git` | `sudo apt install git` / `sudo dnf install git` | [git-scm.com](https://git-scm.com) or `winget install Git.Git` |
| **Bash 4+** (only if you use `bootstrap.sh`) | preinstalled (Bash 3.2 works too) | preinstalled | Git Bash (ships with Git for Windows) or WSL |
| **PowerShell 5.1+ / pwsh 7+** (only for `bootstrap.ps1`) | `brew install --cask powershell` | `sudo snap install powershell --classic` | preinstalled |

Pick **one** runtime: `npx` works everywhere; `bootstrap.sh` for Unix shells; `bootstrap.ps1` for native Windows.

---

## What it ships

```
your-project/
├── AGENTS.md                 # master agent instructions (read by every CLI)
├── CLAUDE.md                 # mirror of AGENTS.md (Claude Code)
├── INIT.md                   # one-shot prompt the agent runs after bootstrap
├── .github/
│   ├── copilot-instructions.md    # mirror of AGENTS.md (Copilot)
│   ├── workflows/                  # CI + Definition-of-Done gate
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── ISSUE_TEMPLATE/
├── .specs/                   # canonical docs (specs as code)
│   ├── product/              # VISION, DOMAIN, PERSONAS
│   ├── architecture/         # DESIGN, PATTERNS, ADRs
│   ├── workflow/             # WORKFLOW, CONTRIBUTING, RELEASE
│   └── sprints/              # BACKLOG + sprint folders
├── .skills/                  # reusable agent skills
├── .agents/                  # custom sub-agents
├── .claude/                  # Claude Code config + hooks
├── .codex/                   # Codex CLI config
├── playwright.config.ts      # default E2E
└── presentation/             # method slides (Marp)
```

Stack-neutral: the bootstrap now fills the first pass automatically from the real project, and `INIT.md` remains available for deeper agent-driven refinement.

---

## Install paths

### A. `npx` — recommended, cross-platform, zero clone

```bash
# inside your project folder (works on macOS, Linux, Windows)
npx @wesleysimplicio/llm-project-mapper
```

Runs interactively. Asks **only**:

1. **Which CLI/LLM to hand off to after the automatic mapping pass** (auto-detects which ones are installed and marks them `[installed]`).
2. **Append recommended ignores to `.gitignore`?** (yes/no — never overwrites your existing `.gitignore`).

Everything else — `PRODUCT_NAME`, stack, dependencies — auto-detected from `package.json` / `pyproject.toml` / `go.mod` / `*.csproj` / `Cargo.toml` / `pubspec.yaml` / `composer.json` / `Gemfile` / `mix.exs` / `pom.xml` / `build.gradle*`.

#### Non-interactive (CI / scripts)

```bash
npx @wesleysimplicio/llm-project-mapper --yes --cli skip --append-gitignore no
```

#### Update an existing starter overlay

```bash
npx @wesleysimplicio/llm-project-mapper@latest --update
```

This is equivalent to `--yes --force --append-gitignore yes --cli skip`: it refreshes starter-managed files, updates the starter `.gitignore` block, preserves existing instruction files, and does not launch an agent.

#### Preview without writing

```bash
npx @wesleysimplicio/llm-project-mapper --dry-run --yes
```

#### Full flag list

| Flag | Purpose |
|---|---|
| `-y, --yes` | Non-interactive (defaults: no `.gitignore` append, skip CLI handoff) |
| `-f, --force` | Overwrite starter template files. **Never** touches user instruction files (`AGENTS.md`, `CLAUDE.md`, `INIT.md`, `.github/copilot-instructions.md`, `.gitignore`) |
| `--update` | Safe update mode for an existing overlay: force starter files, update `.gitignore`, skip handoff |
| `--dry-run` | Print actions without writing |
| `--cli <key>` | Pick CLI for `INIT.md` handoff: `claude`, `codex`, `copilot`, `cursor`, `deepseek`, `kimi`, `minimax`, `glm`, `hermes`, `openclaw`, `aider`, `other`, `skip` |
| `--append-gitignore <yes\|no>` | Append recommended ignores to `.gitignore` |
| `--skip-meta` | Do not write `.starter-meta.json` |
| `--silent` | Minimal output |
| `-v, --version` | Print version |
| `-h, --help` | Show help |

### B. `bootstrap.sh` — Unix shells (macOS / Linux / Git Bash / WSL)

Clone the starter and run the script:

```bash
git clone --depth=1 https://github.com/wesleysimplicio/llm-project-mapper.git tmp-starter
cp -R tmp-starter/. ./ && rm -rf tmp-starter
chmod +x ./bootstrap.sh   # only the first time
./bootstrap.sh
```

### C. `bootstrap.ps1` — native Windows (PowerShell)

```powershell
git clone --depth=1 https://github.com/wesleysimplicio/llm-project-mapper.git tmp-starter
Copy-Item -Recurse -Force tmp-starter\* .\
Remove-Item -Recurse -Force tmp-starter

# PowerShell 7+ (pwsh)
pwsh -File .\bootstrap.ps1

# PowerShell 5.1 (built-in on Windows 10/11)
powershell -ExecutionPolicy Bypass -File .\bootstrap.ps1
```

All three paths produce the same result and ask the same two questions.

### D. Overlay onto an existing project (private, gitignored)

Want to drop the starter into a project that already has its own git, **without polluting the host repo**? Each dev installs locally, files stay gitignored. Full step-by-step in [INSTALL.md](INSTALL.md). Short version:

<img src="assets/overlay-install.svg" alt="Animated terminal screencast showing the overlay install flow in a host project" width="100%">

```bash
# from inside the host project root
git clone --depth=1 https://github.com/wesleysimplicio/llm-project-mapper.git /tmp/llm-project-mapper-src
# --ignore-existing protects your host's package.json/README.md/etc from being overwritten
rsync -av --ignore-existing --exclude='.git' /tmp/llm-project-mapper-src/ ./
rm -rf /tmp/llm-project-mapper-src
# append the "LLM Project Mapper (overlay privado)" block from INSTALL.md to your .gitignore FIRST
# then run bootstrap
./bootstrap.sh
```

---

## CLI handoff — supported agents

After scaffolding and auto-mapping, the bootstrap can optionally launch a CLI/LLM with `INIT.md` for a second-pass refinement. Detected installs get a `[installed]` mark in the menu.

| # | CLI / LLM | Native agent loop? | Install docs |
|---|---|---|---|
| 1 | **Claude Code** | yes | <https://docs.claude.com/claude-code> |
| 2 | **Codex CLI** | yes | <https://github.com/openai/codex> |
| 3 | **GitHub Copilot CLI** | no — paste prompt manually | `gh extension install github/gh-copilot` |
| 4 | **Cursor Agent** | yes | `npm i -g cursor-agent` (or Cursor IDE) |
| 5 | **Deepseek** (via Aider) | yes | `pip install aider-chat` |
| 6 | **Kimi K2.6** (via Aider, OpenRouter) | yes | `pip install aider-chat` |
| 7 | **MiniMax M2.7** (via Aider, OpenRouter) | yes | `pip install aider-chat` |
| 8 | **GLM 5.1** (via Aider, OpenRouter) | yes | `pip install aider-chat` |
| 9 | **Hermes Agent** (Nous Research) | yes | <https://github.com/NousResearch> |
| 10 | **OpenClaw** | yes | <https://github.com/openclaw> |
| 11 | **Aider** (pick model interactively) | yes | `pip install aider-chat` |
| 12 | Other / manual (clipboard) | — | — |
| 13 | Skip — run `INIT.md` later | — | — |

For Copilot CLI (no native agent loop), the bootstrap copies the prompt to your clipboard (`pbcopy` on macOS, `xclip`/`wl-copy` on Linux, `clip.exe` on Windows/WSL) and you paste it into Copilot Chat.

---

## What `INIT.md` does — the safety contract

When the chosen CLI runs `INIT.md`, it reads `.starter-meta.json` and follows three hard rules:

1. **`read_only_globs` are intouchable.** Any file matching these globs (`**/*.razor`, `**/*.cs`, `**/*.csproj`, `**/*.sln`, `package.json`, `pnpm-lock.yaml`, `yarn.lock`, `package-lock.json`, `**/*.py`, `**/*.go`, `**/*.rs`, `**/*.java`, `**/*.kt`, `**/*.dart`, `**/*.php`, `**/*.rb`) is read-only. The agent reads it for context but never writes. If `git status` shows any after init — that is a bug.
2. **`init_must_merge` preserves your essence.** If `AGENTS.md` / `CLAUDE.md` / `.github/copilot-instructions.md` already existed before bootstrap, the agent **reads them**, **preserves the content**, and **merges** the starter structure on top. Never a clean rewrite.
3. **`init_must_ask` only asks 4 things.** `team`, `domain`, `vision_oneliner`, `primary_personas` — once, in a single message. Everything else (`product_name`, `stack`) is auto-detected.

The agent then writes — and only writes — inside the whitelist:

```
.specs/**          .agents/**         .skills/**
.claude/**         .codex/**
.github/copilot-instructions.md
.github/copilot/**
.github/PULL_REQUEST_TEMPLATE.md
.github/ISSUE_TEMPLATE/**
.github/workflows/ci.yml
.github/workflows/dod.yml
AGENTS.md  CLAUDE.md  README.md  README.pt-BR.md
playwright.config.ts (only if missing or our template)
```

Anything outside this whitelist **and** not from the starter template = untouched.

---

## Troubleshooting

### macOS / Linux

| Symptom | Fix |
|---|---|
| `./bootstrap.sh: Permission denied` | `chmod +x ./bootstrap.sh` |
| `command not found: npx` | Install Node.js (see Prerequisites) |
| `Claude Code not installed` after pick | Install Claude Code or pick `[12] Other` to copy the prompt to clipboard |
| Old Bash on macOS (`bash --version` shows 3.2) | Works — script is Bash 3.2-compatible. If problems, `brew install bash` for Bash 5+ |

### Windows

| Symptom | Fix |
|---|---|
| `bootstrap.ps1 cannot be loaded ... execution policy` | Run with `powershell -ExecutionPolicy Bypass -File .\bootstrap.ps1` (per-session bypass, no permanent change) |
| Line endings broken when running `.sh` from Git Bash | `git config --global core.autocrlf input` then re-clone |
| `npx` not found in cmd.exe | Open new terminal after Node install (refreshes PATH), or use full path `C:\Program Files\nodejs\npx.cmd` |
| `pwsh` not found | You have PowerShell 5.1 (built-in) — use the `powershell -ExecutionPolicy Bypass ...` form. To install pwsh 7: `winget install Microsoft.PowerShell` |

### Cross-platform

| Symptom | Fix |
|---|---|
| Bootstrap exits with `aborting: existing files would be overwritten` | Re-run with `--force` (only overwrites starter template files, never your instruction files) |
| `git status` shows `package.json` / source files modified after init | Stop. That is a `read_only_globs` violation. Open an issue with the file path |
| `.gitignore` got rewritten | The starter never overwrites it — only appends if you said `yes`. If yours was replaced, you ran `--force`; restore from git |
| Want to re-run `INIT.md` later | `claude "$(cat INIT.md)"` (or equivalent for your CLI). The handoff is just a launcher |

---

## Suggested reading order (human)

1. `README.md` (this file) — overview.
2. `AGENTS.md` — agent master instruction.
3. `.specs/README.md` — specs navigation map.
4. `.specs/product/VISION.md` — product context.
5. `.specs/architecture/DESIGN.md` — architecture.
6. `.specs/workflow/WORKFLOW.md` — process.
7. `.skills/README.md` — agent capabilities.

---

## Quickstart for the agent (after `INIT.md`)

1. Read `AGENTS.md` (root). That is the contract.
2. Read `.specs/product/VISION.md` for the why.
3. Read `.specs/architecture/DESIGN.md` and `PATTERNS.md` for the how.
4. Pull the next task from `.specs/sprints/sprint-XX/`.
5. Run the mandatory loop: read task → plan → edit → lint → unit → e2e → fix → commit.
6. Validate Definition of Done before opening a PR.

---

## Optional: clean up starter files

After the agent finishes `INIT.md`, the bootstrap files are no longer needed.

**macOS / Linux / Git Bash / WSL:**

```bash
rm _BOOTSTRAP.md INIT.md bootstrap.sh bootstrap.ps1
git add -A && git commit -m "chore: remove starter bootstrap files"
```

**Windows PowerShell:**

```powershell
Remove-Item _BOOTSTRAP.md, INIT.md, bootstrap.sh, bootstrap.ps1
git add -A; git commit -m "chore: remove starter bootstrap files"
```

`.starter-meta.json` stays as a reference for future re-runs.

---

## Companion tooling

- **VS Code extension** — `vscode-extension/` ships a sidebar TreeView for `.specs/sprints/`, plus commands to open the current task, create ADRs, and run the `INIT.md` handoff. See [vscode-extension/README.md](vscode-extension/README.md). Will be published to the Marketplace as `wesleysimplicio.llm-project-mapper-vscode`.
- **Telemetry (opt-in)** — `bin/cli.js` accepts `--telemetry on|off`. Default is off. See [PRIVACY.md](PRIVACY.md) for the exact payload and how to deploy your own [`telemetry-worker.js`](.github/workflows-templates/telemetry-worker.js).

---

## Philosophy

- **Specs as code.** What is not in `.specs/`, the agent does not see.
- **Atomic tasks.** One task = one small reviewable PR.
- **Automated DoD.** What does not pass the gate, does not merge.
- **Reusable skills.** A capability that becomes a pattern becomes a `SKILL.md`.
- **Tight loop.** Edit, test, fix, repeat. Never accumulate invisible debt.
- **Never destroy.** User files are read-only; starter files merge instead of overwrite.

---

## License

[MIT](LICENSE) © 2026 Wesley Simplicio.

---

## Next steps

- Run the bootstrap.
- Let the agent execute `INIT.md`.
- Fill specs with real product context (the agent does most of this from the code).
- Run the first sprint using `.specs/sprints/sprint-01/`.
- Watch `presentation/ai-agent-specialist.pdf` for the full method.
