---
name: lpm-virality
description: pontuar posts para o algoritmo do X (For You) com o engine nativo `lpm virality` — pesos Phoenix, offset de scores negativos, author-diversity decay, fator out-of-network e VQV gating
source: src/virality.rs
---

# Skill: `lpm-virality`

Use o scorer nativo (porta Rust de `x-virality-skills`) para estimar o quão provável um post é de ranquear no feed For You do X, e para raciocinar sobre quais sinais maximizar (reply e follow-author pesam muito; report/block penalizam forte).

> Heurística determinística. Saída numérica idêntica ao `score_simulator.py` (dentro de 1e-9).

---

## Trigger

- Quando o usuário pedir para pontuar/ranquear posts ou comparar candidatos para o X (For You).
- Quando a task envolver "virality", engajamento, pesos do algoritmo, ou otimização de conteúdo para o X/Twitter.
- Quando o pedido mencionar `lpm virality`, `score_simulator`, "Phoenix scores", `vqv`, author-diversity, out-of-network.

---

## Steps

1. Garanta o binário: `cargo build --release`.
2. Monte o input JSON: objeto único ou array de candidatos com `scores` (sinais Phoenix), `in_network`, `video_duration_ms`, `author_id`.
3. Rode: `./target/release/lpm virality --input candidates.json` (texto) ou `--json` (relatórios estruturados).
4. Leia `combined → offset → diversity_adjusted → final` por candidato; use `final` para ranquear.
5. Recomende ações com base nos pesos (priorizar reply/profile-click/follow; evitar sinais que disparam not_interested/report).

---

## Padrões

- Pipeline fixo: `compute_weighted_score → offset → author-diversity → out-of-network`.
- VQV/quoted-VQV só contam se a duração do vídeo exceder `min_video_duration_ms` (5000 ms default).
- `in_network=false` aplica `oon_factor` (0.5) ao score final; autores repetidos sofrem decay de diversidade.
- Pesos default são os do algoritmo público (reply 13.5, follow_author 24.0, profile_click 12.0, report -100.0, …); não invente pesos.
- Evite: tratar o score como verdade absoluta — é heurística para raciocínio, não o ranker de produção.

---

## Definition of Done

- [ ] `cargo build --release` compila sem erro.
- [ ] `lpm virality --input <file>` produz relatório por candidato (ou JSON com `--json`).
- [ ] Ranqueamento por `final` foi usado para a recomendação.
- [ ] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` verdes se `src/virality.rs` foi alterado.

---

## Exemplo

```bash
cargo build --release
cat > candidates.json <<'JSON'
[ { "scores": { "favorite": 0.4, "reply": 0.05, "vqv": 0.3 },
    "in_network": true, "video_duration_ms": 8000, "author_id": 1 } ]
JSON
./target/release/lpm virality --input candidates.json --json
```

---

## Notas

- Módulo: `src/virality.rs`. Origem portada: `src/x_virality_tools/score_simulator.py` (x-virality-skills).
- Paridade numérica verificada contra o Python (combined/offset/diversity/final dentro de 1e-9).
