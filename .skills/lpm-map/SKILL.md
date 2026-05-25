---
name: lpm-map
description: mapear um projeto local com o engine nativo `lpm map` (Rust) para dar contexto ao agente antes de programar — stack, comandos, URLs, domínio, entidades, integrações
source: src/profile.rs, src/detect.rs, src/render.rs
---

# Skill: `lpm-map`

Use o binário nativo `lpm` (Rust) para inspecionar um repositório localmente e gerar o mapa do projeto (`docs/architecture-map.md` + `docs/domain-map.md`) ou um JSON estruturado. É a porta nativa, ~16x mais rápida, do mapper que vivia em `bin/auto-map.js`.

> Engine determinístico, sem rede. Saída byte-a-byte idêntica ao mapper Node.

---

## Trigger

- Quando o usuário pedir para "mapear o projeto", entender stack/comandos/arquitetura de um repo.
- No início de uma task em um repositório desconhecido, antes de editar código.
- Quando precisar de contexto operacional (stack, comandos de dev/test, URLs, integrações) de forma rápida.
- Quando o pedido mencionar `lpm map`, "project map", `architecture-map.md` ou `domain-map.md`.

---

## Steps

1. Garanta o binário: `cargo build --release` (gera `./target/release/lpm`).
2. Rode o mapa: `./target/release/lpm map <path>` (default `.`), que escreve `docs/architecture-map.md` e `docs/domain-map.md`.
3. Para inspeção sem escrever arquivos, use `--dry-run`; para integração de máquina, use `--json`.
4. Leia os artefatos gerados para obter stack, comandos, domínio, entidades e integrações.
5. Use esse contexto para planejar a task antes de tocar no código.

---

## Padrões

- Naming: subcomando `map`; artefatos sempre em `docs/architecture-map.md` e `docs/domain-map.md`.
- `lpm` só sobrescreve docs "starter-managed" (preserva docs do host).
- Prefira `--json` quando outro processo for consumir o mapa; `--dry-run` para só inspecionar.
- Evite: editar os `*-map.md` à mão se o objetivo é o mapa automático — rode `lpm map` de novo.

---

## Definition of Done

- [ ] `cargo build --release` compila sem erro.
- [ ] `lpm map` roda e produz/atualiza os dois `*-map.md` (ou JSON com `--json`).
- [ ] O contexto extraído (stack, comandos) foi usado no planejamento da task.
- [ ] `cargo test`, `cargo clippy -- -D warnings` e `cargo fmt --check` verdes se o engine foi alterado.

---

## Exemplo

```bash
cargo build --release
./target/release/lpm map .                 # escreve docs/architecture-map.md + domain-map.md
./target/release/lpm map /caminho --dry-run # só imprime resumo
./target/release/lpm --json                 # mapa estruturado em JSON
```

---

## Notas

- Módulos: `src/scan.rs`, `src/detect.rs`, `src/profile.rs`, `src/render.rs`; CLI em `src/main.rs`.
- Benchmark: ~4 ms vs ~66 ms (Node) por execução (~16x). Ver README "Benchmark".
- Origem portada: `bin/auto-map.js` (llm-project-mapper).
