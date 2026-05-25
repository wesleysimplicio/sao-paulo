---
name: lpm-yool
description: orquestrar trabalho com o tuple-space / HAMT nativo via `lpm yool` — batch_spawn representa 1.000.000+ subagents virtuais sem enumerar, com receipts content-addressable e snapshot auditável
source: src/yool.rs
---

# Skill: `lpm-yool`

Use o engine nativo YOOL/tuple/HAMT (porta Rust de `simplicio-prompt`) quando precisar raciocinar sobre fan-out hierárquico massivo de subagents sem estourar memória/quota. `batch_spawn(depth, branching)` materializa só um controlador e mantém o resto como contagem virtual + receipts.

> Núcleo determinístico. O runtime de provider (lane pool async, circuit breaker, backoff) fica na spec canônica.

---

## Trigger

- Quando o usuário pedir "1M+ subagents", fan-out hierárquico, ou orquestração estilo swarm sem enumerar.
- Ao decompor uma task em um grafo grande de subagents indexado por Hilbert.
- Quando o pedido mencionar `yool`, `tuple-space`, `HAMT`, `batch_spawn`, `hookwall`, "snapshot".
- Quando a resposta exigir o bloco canônico `[Tuple Space Snapshot] / [Active...] / [Total...] / [Próximo Yool...] / [Resultado parcial]`.

---

## Steps

1. Garanta o binário: `cargo build --release`.
2. Rode o demo/orquestração: `./target/release/lpm yool --depth <D> --branching <B>` (default 4/32 → 1.048.576 virtuais).
3. Use `--threshold N` para o ponto de compressão (compress_token/prune_idle) e `--json` para o snapshot estruturado.
4. Leia o snapshot: `active_agents` (materializados, pequeno) vs `total_agents` (inclui virtuais).
5. Reporte no formato canônico do bloco `[Tuple Space Snapshot]...` quando a task for de orquestração.

---

## Padrões

- `virtual_agents = branching^depth`; só o controlador é materializado por `batch_spawn`.
- `total_agents = active + compressed + virtual`. Nunca enumere os virtuais.
- Receipts são content-addressable (FNV-1a) — estáveis, não criptográficos (não batem byte-a-byte com o blake2b do Python).
- Respeite a lane policy via env `YOOL_TUPLE_*` (ex.: `YOOL_TUPLE_LANE_CONCURRENCY`).
- Evite: criar um agent real por leaf; isso anula o propósito do fan-out lazy.

---

## Definition of Done

- [ ] `cargo build --release` compila sem erro.
- [ ] `lpm yool` imprime o bloco canônico e/ou o snapshot JSON coerente.
- [ ] `active_agents` permanece pequeno enquanto `total_agents` reflete os virtuais.
- [ ] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` verdes se `src/yool.rs` foi alterado.

---

## Exemplo

```bash
cargo build --release
./target/release/lpm yool                          # depth=4, branching=32 -> 1.048.576 virtuais
./target/release/lpm yool --depth 5 --branching 16 --json
```

```text
[Tuple Space Snapshot] 3 tuples, 1 lane(s)
[Active Agents/Subagents] 2
[Total Agents/Subagents] 1048578
[Próximo Yool a executar] codex_worker
[Resultado parcial] batch_spawn@... -> 1048576 virtual subagents (depth=4, branching=32, materialized=2)
```

---

## Notas

- Módulo: `src/yool.rs`. Spec canônica: `YOOL_TUPLE_HAMT.md`.
- Paridade exata de snapshot com `kernel/yool_tuple_kernel.py` (simplicio-prompt).
