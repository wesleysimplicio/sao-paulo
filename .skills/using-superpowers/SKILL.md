---
name: using-superpowers
description: ativar no início de qualquer conversa para descobrir e usar skills relevantes antes de qualquer resposta, inclusive antes de perguntas de esclarecimento
---

# Skill: `using-superpowers`

Esta skill estabelece COMO encontrar e usar as outras skills do catálogo. Antes de responder qualquer coisa — inclusive uma pergunta de esclarecimento — verifique se alguma skill se aplica e invoque-a.

> Se você foi despachado como subagente para executar uma tarefa específica, pule esta skill.

---

## Trigger

- No recebimento de QUALQUER mensagem do usuário (toda mensagem é uma tarefa, inclusive perguntas).
- Antes de entrar em modo de planejamento (plan mode) — se ainda não houve brainstorming, invoque a skill `brainstorming` primeiro.
- Sempre que houver **1% de chance** de uma skill se aplicar à ação que você vai tomar.

---

## Steps

1. Receba a mensagem e pergunte: "alguma skill pode se aplicar aqui?". Se houver até 1% de chance, prossiga para invocar.
2. Invoque a skill via tool `Skill` (no Claude Code). NUNCA leia o arquivo da skill com a tool `Read` — invoque-a de fato.
3. Anuncie em uma linha: "Usando [skill] para [propósito]".
4. Se a skill tiver checklist, crie um item de TodoWrite por item do checklist.
5. Siga a skill exatamente. Se, ao invocá-la, ela se mostrar inadequada à situação, você pode descartá-la.
6. Só então responda (incluindo perguntas de esclarecimento) ou aja.

---

## Padrões

- **Prioridade de instruções** (da mais alta para a mais baixa):
  1. Instruções explícitas do usuário (CLAUDE.md, AGENTS.md, pedidos diretos) — sempre vencem.
  2. Skills — sobrepõem o comportamento padrão do sistema quando há conflito.
  3. Comportamento padrão do system prompt.
  - Se o usuário disser "não use TDD" e uma skill disser "sempre use TDD", siga o usuário. O usuário está no controle.
- **Se uma skill se aplica, NÃO há escolha — você DEVE usá-la.** Isso não é negociável nem opcional; você não pode racionalizar uma saída.
- **Ordem entre skills**: skills de processo primeiro (brainstorming, debugging) — definem COMO abordar a tarefa; skills de implementação depois — guiam a execução. "Vamos construir X" → brainstorming antes; "Conserta esse bug" → debugging antes.
- **Tipos de skill**: skills **rígidas** (TDD, debugging) seguem-se à risca, sem afrouxar a disciplina; skills **flexíveis** (padrões) adaptam-se ao contexto. A própria skill indica qual é.
- Instruções do usuário dizem O QUÊ, não COMO. "Adiciona X" ou "Conserta Y" não significa pular o workflow.

### Red Flags — pensamentos que significam PARE (você está racionalizando)

| Pensamento | Realidade |
| --- | --- |
| "Isso é só uma pergunta simples" | Perguntas são tarefas. Verifique skills. |
| "Preciso de mais contexto antes" | A verificação de skill vem ANTES das perguntas de esclarecimento. |
| "Deixa eu explorar o código primeiro" | Skills dizem COMO explorar. Verifique antes. |
| "Isso não precisa de uma skill formal" | Se existe uma skill, use-a. |
| "Eu lembro dessa skill" | Skills evoluem. Leia a versão atual (invoque). |
| "A skill é exagero pra isso" | Coisas simples viram complexas. Use-a. |
| "Só vou fazer uma coisinha antes" | Verifique ANTES de fazer qualquer coisa. |

---

## Definition of Done

- [ ] A verificação de skills ocorreu ANTES de qualquer resposta ou ação.
- [ ] Toda skill aplicável (≥1% de chance) foi invocada via tool `Skill`, não lida via `Read`.
- [ ] Cada skill invocada foi anunciada ("Usando [skill] para [propósito]").
- [ ] Checklists de skills viraram itens de TodoWrite.
- [ ] Instruções explícitas do usuário foram respeitadas acima das skills em caso de conflito.

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/using-superpowers/SKILL.md`.
- Nomes de tools assumem Claude Code. Em outras plataformas, consulte o mapeamento de tools equivalente (Copilot CLI, Codex, Gemini CLI).
