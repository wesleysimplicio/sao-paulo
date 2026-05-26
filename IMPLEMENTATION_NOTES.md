# São Paulo - Brazilian LLM — Implementation Notes

> Registro vivo das skills ativas e de tudo que implementamos no projeto.
> Por padrão, este arquivo é atualizado a cada modificação (ver "Change log automático").
> Prosa em pt-BR; identificadores/código em inglês.

## Skills ativas / instaladas

Ativadas por padrão no início da sessão (via `.claude/settings.json` SessionStart hook):

- **caveman** — modo terse de resposta (default level `full`); preserva código/commits/PRs em prosa normal.
- **ralph-loop** — loop `read → plan → execute → lint → unit → e2e → fix → repeat` até DoD verde.
- **everything-claude-code** — bundle de agents/skills; reviewers da stack + security-reviewer após edits.

Skills implantadas em `.skills/` (do `llm-project-mapper`, agora commitadas / versionadas):

- Núcleo do método: `caveman`, `ralph-loop`, `everything-claude-code`, `conventional-commits`,
  `playwright-e2e`, `rtk-cli`, `contribute-catalog`, `_template`.
- Animação/web/render: `animejs`, `gsap`, `waapi`, `css-animations`, `lottie`, `tailwind`,
  `three`, `typegpu`, `remotion-to-hyperframes`, `website-to-hyperframes`.
- HyperFrames: `hyperframes`, `hyperframes-cli`, `hyperframes-media`, `hyperframes-registry`.
- **Engine nativo (criadas aqui):** `lpm-map`, `lpm-yool`, `lpm-virality` — expõem os subcomandos
  do binário `lpm` como skills do agente (trigger + steps + DoD), tornando as capacidades nativas.

`.skills/` deixou de ser gitignored (removidas as linhas `.skills/` e `.skills/**`), então as 25
skills passam a versionar no repo. Licenças/atribuição em `.skills/UPSTREAM-LICENSE` e `.skills/NOTICE.md`.

Skills user-invocáveis disponíveis no harness: `update-config`, `verify`, `code-review`,
`security-review`, `review`, `run`, `init`, `loop`, `claude-api`, `session-start-hook`,
`keybindings-help`, `fewer-permission-prompts`.

## Engine nativo `lpm` (Rust)

Binário nativo único (`cargo build --release` → `target/release/lpm`), deps de build: `serde_json`, `blake2`, `sha2`.

| Subcomando | O que faz | Origem portada |
|---|---|---|
| `lpm map [path]` | Mapeia um projeto (stack, comandos, URLs, domínio, entidades, integrações) e gera `docs/architecture-map.md` + `docs/domain-map.md` | `bin/auto-map.js` (llm-project-mapper) |
| `lpm yool [--depth N --branching N]` | Tuple-space / HAMT: `batch_spawn` representa 1M+ subagents virtuais sem enumeração | `kernel/yool_tuple_kernel.py` (simplicio-prompt) |
| `lpm virality --input <file.json>` | Scoring de posts no X (For You): pesos Phoenix, offset, author-diversity, OON, VQV gating | `score_simulator.py` (x-virality-skills) |
| `lpm hamt [root]` | Build do catálogo YOOL/HAMT do `AGENTS.md` (BLAKE2b-64→30bits, HAMT, id=sha256) — sem Python | `scripts/build_hamt.py` |
| `lpm skillopt --input <file.json> [--output skill.md]` | Otimiza um documento de skill para modelo congelado: rollout (offline) → reflect (minibatches sucesso/falha) → edits `add`/`delete`/`replace` com orçamento (learning rate textual) → gate de validação estrito, com buffer de rejeição e slow update por época | loop do SkillOpt (Microsoft Research) |

Módulos: `src/scan.rs`, `src/text.rs`, `src/detect.rs`, `src/profile.rs`, `src/render.rs`,
`src/yool.rs`, `src/virality.rs`, `src/hamt.rs`, `src/skillopt.rs`, CLI em `src/main.rs`, lib em `src/lib.rs`.
Gates: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
Skills nativas: `lpm-map`, `lpm-yool`, `lpm-virality`, `lpm-hamt`, `lpm-skillopt`.

## Histórico de implementação (PRs)

| PR | Entrega | Status |
|---|---|---|
| #1 | Port do runtime nativo US4 V6 Apple Edition (depois substituído) | merged |
| #2 | llm-project-mapper v0.4.2 (ferramenta de mapeamento) | merged |
| #3 | Engine de mapeamento nativo em Rust (`lpm map`), ~16x mais rápido que Node | merged |
| #4 | Rebrand do README para "São Paulo - Brazilian LLM" + seção Benchmark | merged |
| #5 | Port nativo YOOL/tuple/HAMT (`lpm yool`) | merged |
| #6 | Port nativo do scoring x-virality (`lpm virality`) | merged |
| #7 | Port nativo do loop do SkillOpt (`lpm skillopt`) — otimização de skill para modelo congelado | em revisão |

## Benchmark de referência

`lpm map` (Rust nativo) vs mapper Node: **~4 ms vs ~66 ms por execução (~16x)**, binário ~560 KB,
saída byte-a-byte idêntica. `lpm yool` e `lpm virality` com paridade exata contra os kernels Python.

## Como manter este arquivo

- Sempre que uma skill for instalada/ativada ou um recurso for implementado, atualizar as seções acima.
- O hook `PostToolUse` (matcher `Edit|Write`) em `.claude/settings.json` acrescenta uma linha
  timestamped na seção abaixo a cada modificação de arquivo.
- `.claude/settings.json` é commitado (exceção no `.gitignore`: `.claude/*` + `!.claude/settings.json`),
  então o hook é persistente entre sessões/containers. `settings.local.json` e `.claude/hooks/`
  seguem ignorados (estado local).

## Change log automático (hook)

<!-- O hook PostToolUse (Edit|Write) acrescenta entradas "UTC | caminho" abaixo desta linha. -->
2026-05-25T01:18:54Z | /home/user/sao-paulo/.gitignore
2026-05-25T01:19:14Z | /home/user/sao-paulo/IMPLEMENTATION_NOTES.md
2026-05-25T01:20:27Z | /home/user/sao-paulo/.gitignore
2026-05-25T01:20:43Z | /home/user/sao-paulo/IMPLEMENTATION_NOTES.md
2026-05-25T01:24:02Z | /home/user/sao-paulo/.skills/lpm-map/SKILL.md
2026-05-25T01:24:22Z | /home/user/sao-paulo/.skills/lpm-yool/SKILL.md
2026-05-25T01:24:38Z | /home/user/sao-paulo/.skills/lpm-virality/SKILL.md
2026-05-25T01:25:05Z | /home/user/sao-paulo/.skills/README.md
2026-05-25T01:25:19Z | /home/user/sao-paulo/IMPLEMENTATION_NOTES.md
2026-05-25T02:55:56Z | /home/user/sao-paulo/Cargo.toml
2026-05-25T02:57:06Z | /home/user/sao-paulo/src/hamt.rs
2026-05-25T02:57:11Z | /home/user/sao-paulo/src/lib.rs
2026-05-25T02:57:16Z | /home/user/sao-paulo/src/main.rs
2026-05-25T02:57:23Z | /home/user/sao-paulo/src/main.rs
2026-05-25T02:57:28Z | /home/user/sao-paulo/src/main.rs
2026-05-25T02:57:33Z | /home/user/sao-paulo/src/main.rs
2026-05-25T02:57:46Z | /home/user/sao-paulo/src/main.rs
2026-05-25T02:58:23Z | /home/user/sao-paulo/src/hamt.rs
2026-05-25T02:58:28Z | /home/user/sao-paulo/src/hamt.rs
2026-05-25T02:58:39Z | /home/user/sao-paulo/src/hamt.rs
2026-05-25T02:59:22Z | /home/user/sao-paulo/src/hamt.rs
2026-05-25T03:00:19Z | /home/user/sao-paulo/.skills/lpm-hamt/SKILL.md
2026-05-25T03:00:29Z | /home/user/sao-paulo/.skills/README.md
2026-05-25T03:00:44Z | /home/user/sao-paulo/IMPLEMENTATION_NOTES.md
2026-05-26T00:46:50Z | /home/user/sao-paulo/.skills/using-superpowers/SKILL.md
2026-05-26T00:47:00Z | /home/user/sao-paulo/.skills/test-driven-development/SKILL.md
2026-05-26T00:47:10Z | /home/user/sao-paulo/.skills/using-git-worktrees/SKILL.md
2026-05-26T00:47:12Z | /home/user/sao-paulo/.skills/requesting-code-review/SKILL.md
2026-05-26T00:47:16Z | /home/user/sao-paulo/.skills/brainstorming/SKILL.md
2026-05-26T00:47:31Z | /home/user/sao-paulo/.skills/systematic-debugging/SKILL.md
2026-05-26T00:47:36Z | /home/user/sao-paulo/.skills/receiving-code-review/SKILL.md
2026-05-26T00:47:38Z | /home/user/sao-paulo/.skills/finishing-a-development-branch/SKILL.md
2026-05-26T00:47:38Z | /home/user/sao-paulo/.skills/writing-plans/SKILL.md
2026-05-26T00:47:52Z | /home/user/sao-paulo/.skills/verification-before-completion/SKILL.md
2026-05-26T00:47:53Z | /home/user/sao-paulo/.skills/executing-plans/SKILL.md
2026-05-26T00:47:59Z | /home/user/sao-paulo/.skills/dispatching-parallel-agents/SKILL.md
2026-05-26T00:48:03Z | /home/user/sao-paulo/.skills/writing-skills/SKILL.md
2026-05-26T00:48:25Z | /home/user/sao-paulo/.skills/subagent-driven-development/SKILL.md
2026-05-26T00:49:30Z | /home/user/sao-paulo/.skills/README.md
2026-05-26T00:49:39Z | /home/user/sao-paulo/.skills/NOTICE.md
2026-05-26T01:28:43Z | /home/user/sao-paulo/src/skillopt.rs
2026-05-26T01:28:46Z | /home/user/sao-paulo/src/lib.rs
2026-05-26T01:28:51Z | /home/user/sao-paulo/src/main.rs
2026-05-26T01:28:57Z | /home/user/sao-paulo/src/main.rs
2026-05-26T01:29:05Z | /home/user/sao-paulo/src/main.rs
2026-05-26T01:29:08Z | /home/user/sao-paulo/src/main.rs
2026-05-26T01:29:22Z | /home/user/sao-paulo/src/main.rs
2026-05-26T01:29:51Z | /home/user/sao-paulo/src/main.rs
2026-05-26T01:30:24Z | /home/user/sao-paulo/examples/skillopt-rollouts.json
2026-05-26T01:30:54Z | /home/user/sao-paulo/src/skillopt.rs
2026-05-26T01:31:01Z | /home/user/sao-paulo/src/skillopt.rs
2026-05-26T01:31:46Z | /home/user/sao-paulo/README.md
2026-05-26T01:32:13Z | /home/user/sao-paulo/.skills/lpm-skillopt/SKILL.md
2026-05-26T01:32:19Z | /home/user/sao-paulo/.skills/README.md
2026-05-26T01:32:39Z | /home/user/sao-paulo/IMPLEMENTATION_NOTES.md
2026-05-26T01:32:43Z | /home/user/sao-paulo/IMPLEMENTATION_NOTES.md
2026-05-26T01:32:58Z | /home/user/sao-paulo/CHANGELOG.md
2026-05-26T01:34:40Z | /home/user/sao-paulo/.gitignore
