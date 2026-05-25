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

`.skills/` deixou de ser gitignored (removidas as linhas `.skills/` e `.skills/**`), então as 25
skills passam a versionar no repo. Licenças/atribuição em `.skills/UPSTREAM-LICENSE` e `.skills/NOTICE.md`.

Skills user-invocáveis disponíveis no harness: `update-config`, `verify`, `code-review`,
`security-review`, `review`, `run`, `init`, `loop`, `claude-api`, `session-start-hook`,
`keybindings-help`, `fewer-permission-prompts`.

## Engine nativo `lpm` (Rust)

Binário nativo único (`cargo build --release` → `target/release/lpm`), dependência de build: `serde_json`.

| Subcomando | O que faz | Origem portada |
|---|---|---|
| `lpm map [path]` | Mapeia um projeto (stack, comandos, URLs, domínio, entidades, integrações) e gera `docs/architecture-map.md` + `docs/domain-map.md` | `bin/auto-map.js` (llm-project-mapper) |
| `lpm yool [--depth N --branching N]` | Tuple-space / HAMT: `batch_spawn` representa 1M+ subagents virtuais sem enumeração | `kernel/yool_tuple_kernel.py` (simplicio-prompt) |
| `lpm virality --input <file.json>` | Scoring de posts no X (For You): pesos Phoenix, offset, author-diversity, OON, VQV gating | `score_simulator.py` (x-virality-skills) |

Módulos: `src/scan.rs`, `src/text.rs`, `src/detect.rs`, `src/profile.rs`, `src/render.rs`,
`src/yool.rs`, `src/virality.rs`, CLI em `src/main.rs`, lib em `src/lib.rs`.
Gates: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.

## Histórico de implementação (PRs)

| PR | Entrega | Status |
|---|---|---|
| #1 | Port do runtime nativo US4 V6 Apple Edition (depois substituído) | merged |
| #2 | llm-project-mapper v0.4.2 (ferramenta de mapeamento) | merged |
| #3 | Engine de mapeamento nativo em Rust (`lpm map`), ~16x mais rápido que Node | merged |
| #4 | Rebrand do README para "São Paulo - Brazilian LLM" + seção Benchmark | merged |
| #5 | Port nativo YOOL/tuple/HAMT (`lpm yool`) | merged |
| #6 | Port nativo do scoring x-virality (`lpm virality`) | em revisão |

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
