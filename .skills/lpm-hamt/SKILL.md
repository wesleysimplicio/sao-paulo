---
name: lpm-hamt
description: construir o catálogo YOOL/HAMT de agentes a partir do AGENTS.md com o engine nativo `lpm hamt` (Rust) — sem depender de Python
source: src/hamt.rs
---

# Skill: `lpm-hamt`

Use o binário nativo `lpm` para gerar o catálogo de capacidades YOOL/HAMT a partir do `AGENTS.md`. É a porta Rust de `scripts/build_hamt.py` — mesma saída semântica (hashes, slots, árvore HAMT, popcount), só sem dependência de Python.

> Hash: BLAKE2b-64 truncado em 30 bits → slots de 5 bits, 6 níveis. `id` do catálogo = sha256 da serialização canônica.

---

## Trigger

- Quando o usuário pedir para construir/atualizar o catálogo de agentes (`.catalog/agents.json`).
- Ao adicionar/alterar um `### Agent` com `yool_id`/`authority`/`lane`/`agent_terms` no `AGENTS.md`.
- Quando o pedido mencionar `build-hamt-catalog`, `HAMT`, "catálogo YOOL", `agents.json`.

---

## Steps

1. Garanta o binário: `cargo build --release`.
2. Rode: `./target/release/lpm hamt` (default: `AGENTS.md` → `.catalog/agents.json`).
3. Para fontes/saídas custom: `--source <AGENTS.md>` e `--output <path>`.
4. Confira o resumo: `parsed N`, `skipped M`, `root popcount X/32`.
5. Revise `skipped` no JSON: agentes sem `yool_id`/`authority`/`lane`/`agent_terms` ficam de fora.

---

## Padrões

- Só entram agentes com os 4 campos obrigatórios (`yool_id`, `authority`, `lane`, `agent_terms`); o resto vai pra `skipped`.
- Entradas ordenadas por `yool_id`; `id` é determinístico para a mesma entrada (varia só por `generated_at`/`source`).
- Hash é `blake2b-64-truncated-30` — não troque por outra função (quebra o endereçamento de capacidade).
- Evite editar `.catalog/agents.json` à mão; rode `lpm hamt` de novo.

---

## Definition of Done

- [ ] `cargo build --release` compila sem erro.
- [ ] `lpm hamt` escreve o `.catalog/agents.json` e imprime parsed/skipped/popcount.
- [ ] Agentes esperados aparecem em `entries`; incompletos em `skipped`.
- [ ] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` verdes se `src/hamt.rs` mudou.

---

## Exemplo

```bash
cargo build --release
./target/release/lpm hamt                                  # AGENTS.md -> .catalog/agents.json
./target/release/lpm hamt . --source AGENTS.md --output .catalog/agents.json
```

---

## Notas

- Módulo: `src/hamt.rs`. Origem portada: `scripts/build_hamt.py`.
- Paridade semântica verificada contra o Python (mesmos hashes/slots/HAMT/popcount; só `generated_at`/`id`/`source` variam).
- Spec: `YOOL_TUPLE_HAMT.md`.
