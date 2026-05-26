---
name: writing-plans
description: ativar quando houver uma spec ou requisitos para uma tarefa de múltiplos passos, antes de tocar no código, para escrever um plano de implementação detalhado
---

# Skill: `writing-plans`

Escreva planos de implementação completos assumindo que o engenheiro tem zero contexto do codebase e gosto duvidoso. Documente tudo o que ele precisa saber: quais arquivos tocar em cada task, o código, testes, docs a consultar e como testar. Entregue o plano inteiro em tasks pequenas (bite-sized). DRY. YAGNI. TDD. Commits frequentes.

> **Anuncie no início:** "Estou usando a skill writing-plans para criar o plano de implementação."

---

## Trigger

- Quando houver uma spec/design aprovado e a tarefa for de múltiplos passos.
- Logo após o `brainstorming` — é o único próximo passo após o design aprovado.
- Antes de tocar em qualquer código de uma feature multi-step.

---

## Steps

1. **Verifique o escopo**: se a spec cobre múltiplos subsistemas independentes, sugira quebrar em planos separados — um por subsistema, cada um produzindo software funcional e testável por si só.
2. **Mapeie a estrutura de arquivos** antes de definir tasks: quais arquivos serão criados/modificados e a responsabilidade de cada um. Unidades com fronteiras claras, arquivos focados (um arquivo grande costuma fazer coisa demais). Arquivos que mudam juntos ficam juntos; divida por responsabilidade, não por camada técnica.
3. **Escreva o header do plano** (obrigatório): Goal (1 frase), Architecture (2-3 frases), Tech Stack, e a nota para workers indicando a sub-skill de execução.
4. **Decomponha em tasks bottom-up**, cada task com seção **Files** (Create/Modify com path e linhas exatas, Test).
5. **Quebre cada task em steps de ~2-5 min**, um action por step. Padrão TDD: escrever o teste que falha → rodar e ver falhar → implementar o mínimo → rodar e ver passar → commit.
6. **Inclua o conteúdo real em cada step** — código completo em code blocks, comandos exatos com output esperado. Sem placeholders.
7. **Rode a self-review** do plano contra a spec (cobertura, placeholders, consistência de tipos). Corrija inline.
8. **Ofereça a escolha de execução** (subagent-driven recomendado ou execução inline) e invoque a sub-skill de execução escolhida.

---

## Estrutura da task

````markdown
### Task N: [Nome do Componente]

**Files:**
- Create: `exact/path/to/file.py`
- Modify: `exact/path/to/existing.py:123-145`
- Test: `tests/exact/path/to/test.py`

- [ ] **Step 1: Write the failing test**

```python
def test_specific_behavior():
    result = function(input)
    assert result == expected
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pytest tests/path/test.py::test_name -v`
Expected: FAIL with "function not defined"

- [ ] **Step 3: Write minimal implementation**

- [ ] **Step 4: Run test to verify it passes**

Run: `pytest tests/path/test.py::test_name -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/path/test.py src/path/file.py
git commit -m "feat: add specific feature"
```
````

---

## Padrões

- **Granularidade bite-sized**: cada step é UM action (~2-5 min). "Escreva o teste que falha" é um step; "rode e veja falhar" é outro.
- **Sem placeholders** — estes são **falhas de plano**, nunca escreva: "TBD/TODO/implement later"; "adicione tratamento de erro/validação adequados" sem mostrar como; "escreva testes pro acima" sem o código do teste; "similar à Task N" (repita o código — o engenheiro pode ler tasks fora de ordem); referências a tipos/funções não definidos em nenhuma task.
- **Sempre paths de arquivo exatos.** Código completo em todo step que muda código. Comandos exatos com output esperado.
- DRY, YAGNI, TDD, commits frequentes.
- Salve o plano em `docs/plans/YYYY-MM-DD-<feature-name>.md` (preferências do usuário sobre o local prevalecem).

---

## Definition of Done

- [ ] Escopo verificado (decomposto em planos por subsistema, se necessário).
- [ ] Estrutura de arquivos mapeada antes das tasks (responsabilidade clara por arquivo).
- [ ] Header presente (Goal, Architecture, Tech Stack, nota de sub-skill).
- [ ] Tasks decompostas em steps de ~2-5 min, cada um com verificação.
- [ ] Nenhum placeholder; código completo e comandos com output esperado em cada step.
- [ ] Self-review feita: cobertura da spec, placeholders, consistência de tipos.
- [ ] Plano salvo em `docs/plans/...` e escolha de execução oferecida.

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/writing-plans/SKILL.md`.
- Handoff de execução: subagent-driven (um subagente novo por task, review entre tasks) ou execução inline via skill `executing-plans` (execução em lote com checkpoints).
