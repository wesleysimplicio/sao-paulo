---
name: writing-skills
description: criar uma nova skill, editar uma skill existente ou validar uma skill antes de publicar
---

# Skill: `writing-skills`

Escrever uma skill **é TDD aplicado a documentação de processo**. Você roda um cenário-base sem a skill (vê o agente falhar), escreve a skill, roda de novo (vê o agente cumprir) e refatora (fecha brechas). Uma skill é um **guia de referência reutilizável** para uma técnica, padrão ou ferramenta comprovada — não a narrativa de como você resolveu um problema uma vez.

**Lei de ferro:** nenhuma skill sem antes ter visto um agente falhar sem ela.

---

## Trigger

- Ao criar uma nova skill em `.skills/<nome>/SKILL.md`.
- Ao editar uma skill existente (a mesma disciplina vale para edições).
- Ao validar se uma skill funciona antes de publicá-la.

---

## Steps

1. **Decida se vale uma skill.** Crie se a técnica não era óbvia, se aplica amplamente e seria reusada. Não crie para soluções pontuais, práticas já bem documentadas em outro lugar, ou convenções específicas do projeto (essas vão em `CLAUDE.md`/specs).
2. **RED — observe a falha.** Rode o cenário de pressão sem a skill e documente o comportamento e as racionalizações exatas do agente.
3. **GREEN — escreva o mínimo.** Escreva uma skill que ataque *aquelas* falhas específicas. Não adicione conteúdo para casos hipotéticos. Rode o cenário de novo: o agente deve cumprir.
4. **Escreva o frontmatter** com `name` (só letras, números e hífens) e `description` em terceira pessoa, focada **apenas em quando ativar** (gatilhos/sintomas).
5. **Estruture o corpo**: overview com o princípio central, quando usar (e quando NÃO usar), padrão/quick reference, um exemplo concreto, erros comuns.
6. **REFACTOR — feche brechas.** Achou nova racionalização? Adicione um contra-argumento explícito e re-teste até ficar à prova de desculpas.
7. **Valide a Definition of Done** e só então publique. Uma skill por vez — não publique em lote sem testar cada uma.

---

## Padrões

- **`description` = quando usar, NÃO o que a skill faz.** Resumir o workflow na description cria um atalho que o agente segue em vez de ler a skill inteira. Use gatilhos e sintomas, em terceira pessoa, tecnologia-agnósticos (a menos que a skill seja específica).
- **Nome ativo, verbo-primeiro:** `creating-skills` > `skill-creation`; `condition-based-waiting` > `async-test-helpers`. Gerúndios funcionam bem para processos.
- **Responsabilidade única:** uma skill faz uma coisa. Se virar uma lista enorme de convenções, provavelmente é spec, não skill.
- **Steps imperativos** (verbo no infinitivo/imperativo), um por passo. Sem passos compostos.
- **Conciso:** mantenha o essencial inline; mova referência pesada (100+ linhas) ou ferramentas reutilizáveis para arquivos separados na pasta da skill.
- **Um exemplo excelente** vence muitos medíocres. Completo, comentado explicando o *porquê*, de cenário real — não template de preencher-lacunas, não a mesma coisa em 5 linguagens.
- **Definition of Done verificável:** cada item objetivamente checável (true/false).
- **Cross-reference por nome**, com marcador de obrigatoriedade (`**REQUIRED:** ...`). Nunca use `@arquivo` (force-load queima contexto).
- **Sem narrativa.** "Na sessão 2025-10-03 a gente descobriu..." não é skill.
- **Flowchart só** para decisões não-óbvias; nunca para referência (use tabelas), código (use blocos) ou instruções lineares (use lista numerada).

---

## Definition of Done

- [ ] Comportamento-base observado sem a skill (RED) — você viu o agente falhar.
- [ ] `name` só com letras/números/hífens; `description` em terceira pessoa, só com gatilhos (sem resumir o workflow).
- [ ] Overview com princípio central + seção de quando (e quando NÃO) usar.
- [ ] Exatamente um exemplo concreto e completo (não multi-linguagem, não template genérico).
- [ ] Steps imperativos; Definition of Done com itens verificáveis.
- [ ] Cenário re-rodado COM a skill — o agente cumpre (GREEN).
- [ ] Brechas/racionalizações novas fechadas com contra-argumento explícito (REFACTOR).
- [ ] Sem narrativa, sem `@` links, referência pesada movida para arquivo separado.

---

## Exemplo

```yaml
# ❌ RUIM: resume o workflow — o agente segue isso em vez de ler a skill
description: Use ao executar planos — despacha um subagente por task com review entre tasks

# ❌ RUIM: vago e em primeira pessoa
description: posso te ajudar com testes async quando estão flaky

# ✅ BOM: só condições de ativação, sem resumo de processo, em terceira pessoa
description: usar quando testes têm race conditions, dependências de timing ou passam/falham de forma inconsistente
```

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/writing-skills/SKILL.md` (distilada — o original é bem mais extenso).
- O upstream apoia-se em `test-driven-development` como pré-requisito e detalha a metodologia de teste com subagentes em arquivos de apoio.
- Spec do frontmatter: <https://agentskills.io/specification> (frontmatter ≤ 1024 chars).
- Neste projeto, skills vivem em `.skills/<nome>/SKILL.md` — use `_template/SKILL.md` como base e siga o formato da casa.
