---
name: verification-before-completion
description: ative quando estiver prestes a declarar trabalho concluído, corrigido ou passando, antes de commitar ou abrir PR — rode o comando de verificação e confirme o output antes de qualquer afirmação de sucesso
---

# Skill: `verification-before-completion`

Afirmar que o trabalho está concluído sem verificar é desonestidade, não eficiência. **Princípio central:** evidência antes de afirmações, sempre.

> **Violar a letra desta regra é violar o espírito desta regra.**

---

## A Lei de Ferro

```
NENHUMA AFIRMAÇÃO DE CONCLUSÃO SEM EVIDÊNCIA DE VERIFICAÇÃO FRESCA
```

Se você não rodou o comando de verificação **nesta mensagem**, você não pode afirmar que passa.

---

## Trigger

Ative SEMPRE antes de:
- Qualquer variação de afirmação de sucesso/conclusão ("pronto", "corrigido", "passando").
- Qualquer expressão de satisfação ("Ótimo!", "Perfeito!", "Feito!").
- Qualquer afirmação positiva sobre o estado do trabalho.
- Commitar, abrir PR, concluir tarefa, avançar para a próxima tarefa, delegar para agentes.

A regra se aplica a frases exatas, paráfrases, sinônimos e qualquer implicação de sucesso.

---

## Steps — A Função de Portão (Gate)

Antes de afirmar qualquer status ou expressar satisfação:

1. **IDENTIFIQUE:** qual comando prova esta afirmação?
2. **RODE:** execute o comando COMPLETO (fresco, do início).
3. **LEIA:** o output inteiro; cheque o exit code; conte as falhas.
4. **VERIFIQUE:** o output confirma a afirmação?
   - Se NÃO: declare o status real **com a evidência**.
   - Se SIM: faça a afirmação **junto com a evidência**.
5. **SÓ ENTÃO:** faça a afirmação.

Pular qualquer passo = mentir, não verificar.

---

## Padrões

Cada afirmação exige uma evidência específica — e o que **não** basta:

| Afirmação | Exige | Não basta |
|---|---|---|
| Testes passam | Output do comando de teste: 0 falhas | Run anterior, "deveria passar" |
| Linter limpo | Output do linter: 0 erros | Checagem parcial, extrapolação |
| Build OK | Comando de build: exit 0 | Linter passando, "logs parecem bons" |
| Bug corrigido | Testar o sintoma original: passa | Código mudou, "presumo corrigido" |
| Teste de regressão funciona | Ciclo red-green verificado | Teste passou uma vez |
| Agente concluiu | Diff do VCS mostra as mudanças | Agente reportou "sucesso" |
| Requisitos atendidos | Checklist linha por linha | Testes passando |

Exemplos de padrão correto vs. errado:

```
✅ [rodar comando de teste] [ver: 34/34 pass] "Todos os testes passam"
❌ "Deveria passar agora" / "Parece correto"

# Teste de regressão (TDD red-green):
✅ Escrever → Rodar (passa) → Reverter o fix → Rodar (DEVE FALHAR) → Restaurar → Rodar (passa)
❌ "Escrevi um teste de regressão" (sem o ciclo red-green)

# Build:
✅ [rodar build] [ver: exit 0] "Build passa"
❌ "Linter passou" (linter não checa compilação)

# Delegação a agente:
✅ Agente reporta sucesso → checar diff do VCS → verificar mudanças → reportar estado real
❌ Confiar no relatório do agente
```

---

## Red Flags — PARE

- Usar "deveria", "provavelmente", "parece que".
- Expressar satisfação antes da verificação.
- Estar prestes a commitar/push/PR sem verificar.
- Confiar em relatório de sucesso de agente.
- Apoiar-se em verificação parcial.
- Pensar "só dessa vez" ou estar cansado e querendo terminar.
- Qualquer redação que implique sucesso sem ter rodado a verificação.

Racionalizações e a realidade: "deveria funcionar agora" → RODE a verificação; "estou confiante" → confiança ≠ evidência; "linter passou" → linter ≠ compilador; "o agente disse sucesso" → verifique de forma independente; "checagem parcial basta" → parcial não prova nada; "palavras diferentes, então a regra não se aplica" → espírito acima da letra.

---

## Definition of Done

- [ ] Identifiquei o comando que prova cada afirmação de sucesso.
- [ ] Rodei o comando COMPLETO e fresco nesta mensagem (não reusei run anterior).
- [ ] Li o output inteiro, conferi o exit code e contei as falhas.
- [ ] Toda afirmação de sucesso vem acompanhada da evidência correspondente.
- [ ] Teste de regressão (se houver) verificado pelo ciclo red-green.
- [ ] Para trabalho delegado: conferi o diff do VCS em vez de confiar no relatório.
- [ ] Requisitos checados linha por linha contra o plano/checklist.

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/verification-before-completion/SKILL.md`.
- Sem atalhos para a verificação: rode o comando, leia o output, ENTÃO afirme o resultado. Não negociável.
- Relacionada: `test-driven-development` (ciclo red-green) e `systematic-debugging` (verificar que o fix funcionou).
