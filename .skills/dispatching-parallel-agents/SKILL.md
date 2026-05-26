---
name: dispatching-parallel-agents
description: usar quando há 2+ tarefas independentes que podem ser trabalhadas sem estado compartilhado nem dependências sequenciais — despacha um agente por domínio de problema, em paralelo
---

# Skill: `dispatching-parallel-agents`

Você delega tarefas a agentes especializados com contexto isolado. Ao construir instruções e contexto com precisão, garante que eles fiquem focados e tenham sucesso. Eles **nunca** herdam o contexto ou histórico da sua sessão — você monta exatamente o que cada um precisa. Isso também preserva seu próprio contexto para o trabalho de coordenação.

**Princípio central:** despache um agente por domínio de problema independente. Deixe-os trabalhar simultaneamente.

---

## Trigger

- 3+ arquivos de teste falhando com causas-raiz diferentes.
- Múltiplos subsistemas quebrados de forma independente.
- Cada problema pode ser entendido sem contexto dos outros.
- Sem estado compartilhado entre as investigações.

**Não use quando:**

- As falhas são relacionadas (corrigir uma pode corrigir as outras).
- Você precisa entender o estado completo do sistema.
- Os agentes interfeririam entre si (editando os mesmos arquivos, usando os mesmos recursos).
- Debugging exploratório — você ainda não sabe o que está quebrado.

---

## Steps

1. **Identifique domínios independentes.** Agrupe as falhas pelo que está quebrado. Cada domínio deve ser independente — corrigir um não afeta o outro.
2. **Crie tarefas focadas por agente.** Cada agente recebe:
   - **Escopo específico:** um arquivo de teste ou subsistema.
   - **Objetivo claro:** "faça estes testes passarem".
   - **Restrições:** "não mude outro código".
   - **Output esperado:** resumo do que encontrou e corrigiu.
3. **Despache em paralelo.** Lance todos os agentes concorrentemente — em uma única leva de chamadas, não uma após a outra:
   ```typescript
   // No ambiente Claude Code / agente
   Task("Fix agent-tool-abort.test.ts failures")
   Task("Fix batch-completion-behavior.test.ts failures")
   Task("Fix tool-approval-race-conditions.test.ts failures")
   // Os três rodam concorrentemente
   ```
4. **Revise e integre.** Quando os agentes retornarem: leia cada resumo, verifique se as correções conflitam, rode a suíte completa e integre todas as mudanças.

---

## Padrões

Bons prompts de agente são:

1. **Focados** — um domínio de problema claro.
2. **Auto-contidos** — todo o contexto necessário para entender o problema (cole as mensagens de erro e os nomes dos testes; o agente não vê sua sessão).
3. **Específicos sobre o output** — o que o agente deve retornar.

Exemplo de prompt:

```markdown
Fix the 3 failing tests in src/agents/agent-tool-abort.test.ts:

1. "should abort tool with partial output capture" - expects 'interrupted at' in message
2. "should handle mixed completed and aborted tools" - fast tool aborted instead of completed
3. "should properly track pendingToolCount" - expects 3 results but gets 0

These are timing/race condition issues. Your task:

1. Read the test file and understand what each test verifies
2. Identify root cause - timing issues or actual bugs?
3. Fix by:
   - Replacing arbitrary timeouts with event-based waiting
   - Fixing bugs in abort implementation if found
   - Adjusting test expectations if testing changed behavior

Do NOT just increase timeouts - find the real issue.

Return: Summary of what you found and what you fixed.
```

Erros comuns:

- **Amplo demais:** "conserta todos os testes" → o agente se perde. **Específico:** "conserta `agent-tool-abort.test.ts`".
- **Sem contexto:** "conserta a race condition" → o agente não sabe onde. **Com contexto:** cole erros e nomes de teste.
- **Sem restrições:** o agente pode refatorar tudo. **Com restrições:** "NÃO mude código de produção".
- **Output vago:** "conserta" → você não sabe o que mudou. **Específico:** "retorne resumo da causa-raiz e das mudanças".

---

## Definition of Done

- [ ] Falhas agrupadas em domínios comprovadamente independentes.
- [ ] Um agente por domínio, cada um com escopo, objetivo, restrições e output definidos.
- [ ] Prompts auto-contidos (erros e nomes de teste colados, sem depender da sessão do controlador).
- [ ] Agentes despachados em paralelo, não sequencialmente.
- [ ] Cada resumo lido e verificado contra conflitos entre as mudanças.
- [ ] Suíte de testes completa rodada após a integração e passando.

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/dispatching-parallel-agents/SKILL.md`.
- Benefícios: paralelização, foco (escopo estreito por agente), independência e velocidade (N problemas resolvidos no tempo de 1).
- Spot check: agentes podem cometer erros sistemáticos — sempre revise os resumos e rode a suíte completa antes de declarar pronto.
