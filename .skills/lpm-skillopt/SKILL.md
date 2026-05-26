---
name: lpm-skillopt
description: otimizar um documento de skill (linguagem natural) para um modelo congelado com o engine nativo `lpm skillopt` — porta do loop do SkillOpt (Microsoft Research): rollout → reflect (minibatches de sucesso/falha) → edits add/delete/replace com orçamento (learning rate textual) → gate de validação estrito, com buffer de rejeição e slow update por época
source: src/skillopt.rs
---

# Skill: `lpm-skillopt`

Use o otimizador nativo (porta Rust do [SkillOpt](https://microsoft.github.io/SkillOpt/)) para **treinar um documento de skill como se treina uma rede** — épocas, minibatch, learning rate e gate de validação — **sem tocar nos pesos do modelo**. O artefato treinável é o próprio markdown da skill; o engine propõe edits limitados (`add`/`delete`/`replace`) a partir de rollouts pontuados e só aceita um edit quando ele **melhora estritamente** a pontuação de validação held-out.

> Determinístico. O *control plane* (loop, orçamento, gate, buffer, slow update) é nativo; o passo de reflect que um modelo otimizador faria é modelado por recorrência sobre rollouts offline. Trocar por um modelo real substitui só a heurística de reflect — o resto do loop é idêntico.

---

## Trigger

- Quando o usuário pedir para **otimizar / evoluir / treinar uma skill** (documento de instruções) sem fine-tuning.
- Quando a task mencionar `SkillOpt`, "self-evolving skills", "skill como estado treinável", learning rate textual, edit budget, gate de validação.
- Quando houver rollouts pontuados (sucesso/falha por task) e a meta for descobrir quais procedimentos a skill deve conter.
- Quando o pedido mencionar `lpm skillopt`.

---

## Steps

1. Garanta o binário: `cargo build --release`.
2. Monte o input JSON com três chaves:
   - `skill`: `{ title, sections?, lessons }` — `lessons` é o estado inicial (chaves de procedimento ativas).
   - `tasks`: array de rollouts pontuados, cada um com `split` (`train`/`val`), `success` (ou `score` numérico ≥ 0.5) e `requires` (chaves de procedimento que a task precisa).
   - `catalog`: mapa `chave -> texto do bullet` (ou `{ text, supersedes }` para habilitar `replace`).
3. Rode: `./target/release/lpm skillopt --input rollouts.json --output skill.md` (resumo + escreve o markdown otimizado) ou `--json` (histórico completo de treino + resultado).
4. Leia `val inicial → val final` e a lista de **procedimentos finais**; use `skill.md` como a skill implantável.
5. Ajuste hiperparâmetros se necessário: `--epochs`, `--batch-size`, `--edit-budget` (learning rate textual), `--gate-margin`, `--slow-cap`.

---

## Padrões

- Loop fixo: `rollout (offline) → reflect (minibatches de sucesso/falha separados) → edit limitado → gate`.
- O **edit budget** é o learning rate textual: limita add/replace por candidato (default 3). Mais alto = passos maiores, mais risco.
- O **gate** aceita um candidato só se a validação **melhora estritamente** (`> base + gate-margin`); senão as chaves vão pro **buffer de rejeição** e não são repropostas.
- O **slow update** (fim de época) poda procedimentos obsoletos (não exigidos por nenhuma task) sem deixar a validação cair.
- `replace`: uma chave de catálogo com `supersedes: <antiga>` substitui a antiga (1 edit) e ainda cobre tasks que exigiam a antiga.
- Saída é **um único markdown implantável** — o entregável do SkillOpt. Não invente procedimentos fora do `catalog`/`requires`.
- Evite: tratar o score de validação como verdade de produção — é proxy de cobertura para raciocínio determinístico, não a métrica do paper.

---

## Definition of Done

- [ ] `cargo build --release` compila sem erro.
- [ ] `lpm skillopt --input <file>` roda e reporta `val inicial → val final` (ou JSON com `--json`).
- [ ] `--output <file.md>` gravou a skill otimizada com a seção "Procedimentos aprendidos (SkillOpt)".
- [ ] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` verdes se `src/skillopt.rs` foi alterado.

---

## Exemplo

```bash
cargo build --release
cat > rollouts.json <<'JSON'
{
  "skill": { "title": "Code agent", "lessons": [] },
  "catalog": {
    "abs-paths": "Use caminhos absolutos ao editar arquivos.",
    "verify": "Rode a verificação antes de concluir."
  },
  "tasks": [
    { "split": "train", "success": false, "requires": ["abs-paths"] },
    { "split": "train", "success": false, "requires": ["abs-paths", "verify"] },
    { "split": "val",   "success": false, "requires": ["abs-paths"] },
    { "split": "val",   "success": false, "requires": ["verify"] }
  ]
}
JSON
./target/release/lpm skillopt --input rollouts.json --output skill.md
./target/release/lpm skillopt --input rollouts.json --json
```

Um dataset maior já vendorizado: `examples/skillopt-rollouts.json`.

---

## Notas

- Módulo: `src/skillopt.rs`. Origem: loop do **SkillOpt** (Microsoft Research, https://microsoft.github.io/SkillOpt/, arXiv 2605.23904).
- Esta é a porta determinística do *control plane* (loop/orçamento/gate/buffer/slow update). O passo de reflect do modelo otimizador é representado por recorrência sobre rollouts offline; o gate, o orçamento e o buffer são fiéis ao método.
