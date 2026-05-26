---
name: receiving-code-review
description: receber feedback de code review, antes de implementar sugestões, especialmente quando o feedback parecer vago ou tecnicamente questionável — exige rigor técnico e verificação, não concordância performática nem implementação cega
---

# Skill: `receiving-code-review`

Code review pede **avaliação técnica, não performance emocional**. Verifique antes de implementar. Pergunte antes de assumir. Correção técnica acima de conforto social.

**Princípio central:** feedback externo são *sugestões a avaliar*, não *ordens a seguir*.

---

## Trigger

- Ao receber feedback de code review (humano ou revisor externo/automático).
- Antes de implementar qualquer sugestão de revisão.
- Quando o feedback parecer vago, incompleto ou tecnicamente duvidoso.
- Ao responder comentários inline em PR no GitHub.

---

## Steps

1. **Leia** o feedback completo sem reagir.
2. **Entenda**: reformule cada requisito com suas palavras (ou pergunte se não entendeu).
3. **Verifique** contra a realidade do codebase (`grep`, leitura, testes) — a sugestão é correta para *este* projeto?
4. **Avalie**: quebra algo existente? Há razão para a implementação atual? Funciona em todas as plataformas/versões? O revisor tem o contexto completo?
5. **Responda**: reconhecimento técnico ou pushback fundamentado. Se algo estiver pouco claro, **PARE** e peça esclarecimento antes de implementar qualquer item.
6. **Implemente** um item por vez, na ordem: bloqueadores (quebras, segurança) → correções simples (typos, imports) → correções complexas (refactor, lógica). Teste cada um isoladamente.

---

## Padrões

- **Concordância performática é proibida.** Nunca escreva "você está certíssimo", "ótimo ponto", "obrigado por pegar isso" nem qualquer agradecimento. Ações falam — apenas corrija e mostre no código.
- **Não implemente cego.** Sempre verifique contra o codebase antes.
- **Esclareça tudo antes de começar** quando há múltiplos itens — itens podem ser relacionados, e entendimento parcial gera implementação errada.
- **Check de YAGNI**: se o revisor sugere "implementar direito" uma feature, faça `grep` por uso real. Sem uso → proponha remover (YAGNI). Com uso → implemente direito.
- **Não consegue verificar?** Diga: "não consigo verificar isso sem [X]. Devo investigar / perguntar / prosseguir?".
- **Conflito com decisões arquiteturais já tomadas?** Pare e discuta antes.

---

## Quando dar pushback

Dê pushback (com raciocínio técnico, não defensividade) quando a sugestão:
- Quebra funcionalidade existente.
- Vem de revisor sem o contexto completo.
- Viola YAGNI (feature sem uso).
- É tecnicamente incorreta para esta stack.
- Ignora razões de legado/compatibilidade.
- Conflita com decisões arquiteturais já tomadas.

Use raciocínio técnico, faça perguntas específicas, referencie testes/código que funcionam.

**Se deu pushback e estava errado:** corrija de forma factual e siga — "Verifiquei e você está certo, meu entendimento inicial estava errado porque [motivo]. Corrigindo." Sem desculpas longas nem defesa do erro.

---

## Definition of Done

- [ ] Cada item de feedback foi entendido (reformulado) ou esclarecido antes de implementar.
- [ ] Sugestões verificadas contra o codebase real (não assumidas como corretas).
- [ ] Itens implementados um a um, testados isoladamente, sem regressão.
- [ ] Pushback (quando aplicável) feito com raciocínio técnico, não com defensividade.
- [ ] Nenhuma resposta performática nem agradecimento — só o estado técnico das correções.

---

## Exemplos

**Concordância performática (ruim):**
```
Revisor: "Remova o código legado"
❌ "Você está certíssimo! Vou remover já..."
```

**Verificação técnica (bom):**
```
Revisor: "Remova o código legado"
✅ "Checando... o build alvo é 10.15+, essa API exige 13+. Preciso do legado para
   compat. A impl atual está com o bundle ID errado — corrijo isso ou dropamos
   suporte a pré-13?"
```

**YAGNI (bom):**
```
Revisor: "Implemente métricas completas com banco, filtros de data e export CSV"
✅ "Fiz grep no codebase — nada chama esse endpoint. Removo (YAGNI)? Ou tem uso
   que eu não vi?"
```

**Item pouco claro (bom):**
```
Parceiro: "Corrija os itens 1-6"
Entendo 1,2,3,6. Não entendi 4,5.
✅ "Entendi 1,2,3,6. Preciso de esclarecimento em 4 e 5 antes de implementar."
```

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/receiving-code-review/SKILL.md`.
- Complementar a `requesting-code-review` (como solicitar a revisão).
- No GitHub, responda comentários inline **na thread do comentário** (`gh api repos/{owner}/{repo}/pulls/{pr}/comments/{id}/replies`), não como comentário de topo do PR.
