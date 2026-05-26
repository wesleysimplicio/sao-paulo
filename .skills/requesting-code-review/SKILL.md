---
name: requesting-code-review
description: solicitar revisão de código por um agente independente ao concluir uma task, finalizar uma feature grande ou antes de fazer merge, verificando se o trabalho atende aos requisitos/spec
---

# Skill: `requesting-code-review`

Despache um subagente revisor para pegar problemas antes que se propaguem. O revisor recebe **apenas o contexto necessário** para avaliar o trabalho — nunca o histórico da sua sessão. Isso o mantém focado no produto final, não no seu raciocínio, e preserva o seu próprio contexto.

**Princípio central:** revise cedo, revise sempre.

---

## Trigger

**Obrigatório:**
- Após cada task em desenvolvimento orientado por subagentes.
- Ao concluir uma feature grande.
- Antes de fazer merge na branch principal.

**Opcional mas valioso:**
- Quando estiver travado (perspectiva nova).
- Antes de um refactor (checagem de baseline).
- Após corrigir um bug complexo.

---

## Steps

1. **Pegue os SHAs do git** que delimitam a mudança:
   ```bash
   BASE_SHA=$(git rev-parse HEAD~1)   # ou origin/main
   HEAD_SHA=$(git rev-parse HEAD)
   ```
2. **Despache um subagente revisor** (agente independente, sem o histórico da sua sessão).
3. **Forneça o contexto preciso** ao revisor: descrição curta do que foi construído, o plano/requisitos que deveria atender, e os SHAs `BASE_SHA` e `HEAD_SHA`.
4. **Instrua o revisor a avaliar contra a spec/requisitos**, não contra suposições — ele deve confirmar se o trabalho realmente faz o que foi pedido.
5. **Aja sobre o feedback**: corrija issues *Critical* imediatamente; corrija *Important* antes de prosseguir; anote *Minor* para depois.
6. **Discorde com fundamento técnico** se o revisor estiver errado (mostre o código/teste que prova que funciona).

---

## Padrões

- O revisor recebe contexto **montado de propósito**, nunca o transcript da sessão — isso evita viés e economiza contexto.
- Classifique sempre os achados em três níveis: **Critical**, **Important**, **Minor**.
- A revisão é contra **o que deveria ser feito** (plano/requisitos), não contra o que você acha que fez.
- Não pule revisão por "é simples demais". Simples também quebra.
- Não prossiga com issues *Important* em aberto.

---

## Definition of Done

- [ ] `BASE_SHA` e `HEAD_SHA` definidos corretamente delimitando a mudança.
- [ ] Subagente revisor despachado com descrição + requisitos/spec + SHAs.
- [ ] Feedback recebido e classificado (Critical / Important / Minor).
- [ ] Issues *Critical* e *Important* corrigidos antes de seguir.
- [ ] Discordâncias do revisor respondidas com raciocínio técnico (não ignoradas).

---

## Exemplo

```
[Acabei a Task 2: adicionar função de verificação]

Eu: Vou solicitar revisão antes de prosseguir.

BASE_SHA=a7981ec
HEAD_SHA=3df7661

[Despacha subagente revisor]
  DESCRIPTION: adicionado verifyIndex() e repairIndex() com 4 tipos de issue
  PLAN_OR_REQUIREMENTS: Task 2 do plano de deployment
  BASE_SHA: a7981ec
  HEAD_SHA: 3df7661

[Subagente retorna]:
  Pontos fortes: arquitetura limpa, testes reais
  Issues:
    Important: faltam indicadores de progresso
    Minor: número mágico (100) no intervalo de report
  Avaliação: pronto para prosseguir

Eu: [corrijo os indicadores de progresso]
[Sigo para a Task 3]
```

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/requesting-code-review/SKILL.md`.
- Complementar a `receiving-code-review` (como lidar com o feedback recebido).
- O upstream usa um template `code-reviewer.md` para preencher os placeholders do prompt do revisor.
