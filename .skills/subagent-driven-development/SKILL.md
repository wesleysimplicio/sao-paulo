---
name: subagent-driven-development
description: usar quando for executar um plano de implementação com tarefas independentes na sessão atual — despacha um subagente novo por tarefa, com revisão em dois estágios (spec depois qualidade) após cada uma
---

# Skill: `subagent-driven-development`

Execute o plano despachando um subagente novo por tarefa, com revisão em dois estágios após cada uma: primeiro conformidade com a spec, depois qualidade de código.

**Por que subagentes:** você delega a agentes especializados com contexto isolado. Eles nunca herdam o contexto ou histórico da sua sessão — você monta exatamente o que precisam. Isso preserva seu próprio contexto para coordenação.

**Princípio central:** subagente novo por tarefa + revisão em dois estágios (spec, depois qualidade) = alta qualidade e iteração rápida.

**Execução contínua:** não pare para checar com o humano entre tarefas. Execute todas as tarefas do plano sem parar. Os únicos motivos para parar são: status BLOCKED que você não consegue resolver, ambiguidade que genuinamente impede o progresso, ou todas as tarefas concluídas.

---

## Trigger

- Quando você tem um plano de implementação com tarefas majoritariamente independentes.
- Quando vai executar o plano na **sessão atual** (sem trocar de sessão).
- Quando o usuário pedir "executa o plano", "implementa essas tarefas".

**Não use quando** as tarefas são fortemente acopladas (faça execução manual ou brainstorm primeiro) ou quando for executar em sessão paralela (use a skill de executing-plans).

---

## Steps

1. **Leia o plano uma vez.** Extraia todas as tarefas com o texto completo, anote o contexto e crie um TodoWrite com todas elas. O controlador fornece o texto — os subagentes **não** leem o arquivo de plano.
2. **Por tarefa: despache o subagente implementador** com o texto completo da tarefa + contexto (scene-setting de onde ela se encaixa). Use o modelo menos potente que dê conta (ver Padrões).
3. **Responda às perguntas do implementador**, se houver, antes de deixá-lo prosseguir. Re-despache com o contexto adicional.
4. **Deixe o implementador implementar, testar, comitar e fazer self-review.** Trate o status reportado:
   - **DONE:** siga para a revisão de spec.
   - **DONE_WITH_CONCERNS:** leia as ressalvas; resolva as de correção/escopo antes de revisar, anote as observacionais e siga.
   - **NEEDS_CONTEXT:** forneça o contexto faltante e re-despache.
   - **BLOCKED:** avalie — problema de contexto (forneça mais e re-despache mesmo modelo); precisa de mais raciocínio (modelo mais capaz); tarefa grande demais (quebre em pedaços); plano errado (escale ao humano). **Nunca** force o mesmo modelo a retentar sem nenhuma mudança.
5. **Estágio 1 — Despache o revisor de spec.** Ele confirma se o código bate com a spec (nada faltando, nada extra). Se achar problemas, o **mesmo** implementador corrige e o revisor revisa de novo. Repita até ✅.
6. **Estágio 2 — Despache o revisor de qualidade de código** (só **depois** do estágio 1 estar ✅). Se achar problemas, o implementador corrige e o revisor revisa de novo. Repita até aprovado.
7. **Marque a tarefa como completa no TodoWrite** e volte ao passo 2 para a próxima tarefa.
8. **Após todas as tarefas:** despache um revisor de código final para a implementação inteira.
9. **Finalize** usando a skill `finishing-a-development-branch`.

---

## Padrões

- **Ordem dos estágios é inviolável:** conformidade com a spec **primeiro**, qualidade de código **depois**. Nunca inicie a revisão de qualidade antes da spec estar ✅.
- **Loop de revisão obrigatório:** revisor achou problema → implementador (mesmo subagente) corrige → revisor revisa de novo. Nunca pule a re-revisão nem aceite "perto o suficiente".
- **Um implementador por vez:** nunca despache múltiplos subagentes implementadores em paralelo (conflitos).
- **Não faça o subagente ler o plano** — forneça o texto completo da tarefa + contexto.
- O self-review do implementador **não** substitui a revisão real — ambos são necessários.
- **Seleção de modelo:** tarefa mecânica (1-2 arquivos, spec completa) → modelo rápido/barato; integração/julgamento (múltiplos arquivos) → modelo padrão; arquitetura/design/revisão → modelo mais capaz.
- **Nunca** inicie implementação em main/master sem consentimento explícito do usuário.
- Se um subagente falhar a tarefa, despache um subagente de correção com instruções específicas — não conserte manualmente (poluição de contexto).

---

## Definition of Done

- [ ] Plano lido uma vez; todas as tarefas extraídas com texto completo no TodoWrite.
- [ ] Cada tarefa executada por um subagente implementador novo, com contexto auto-contido.
- [ ] Perguntas do implementador respondidas antes de prosseguir.
- [ ] Estágio 1 (spec) ✅ antes de iniciar o estágio 2 (qualidade).
- [ ] Loops de revisão fechados — nenhum problema aberto em qualquer estágio.
- [ ] Nenhum implementador despachado em paralelo com outro.
- [ ] Revisor de código final rodado sobre a implementação inteira.
- [ ] Conclusão delegada à skill `finishing-a-development-branch`.

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/subagent-driven-development/SKILL.md`.
- Templates de prompt do upstream: `implementer-prompt.md`, `spec-reviewer-prompt.md`, `code-quality-reviewer-prompt.md`.
- Skills relacionadas: `using-git-worktrees` (workspace isolado no início), `finishing-a-development-branch` (conclusão). Subagentes devem seguir TDD em cada tarefa.
