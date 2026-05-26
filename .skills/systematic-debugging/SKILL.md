---
name: systematic-debugging
description: ative ao encontrar qualquer bug, falha de teste ou comportamento inesperado, antes de propor qualquer correção — sempre encontre a causa raiz antes de corrigir
---

# Skill: `systematic-debugging`

Correções aleatórias desperdiçam tempo e criam novos bugs. Patches rápidos mascaram o problema real. **Princípio central:** SEMPRE encontre a causa raiz antes de tentar qualquer fix. Corrigir o sintoma é fracasso.

> **Violar a letra deste processo é violar o espírito do debugging.**

---

## A Lei de Ferro

```
NENHUMA CORREÇÃO SEM INVESTIGAR A CAUSA RAIZ PRIMEIRO
```

Se você não completou a Fase 1, você não pode propor correções.

---

## Trigger

Use para QUALQUER problema técnico: falha de teste, bug em produção, comportamento inesperado, problema de performance, falha de build, problema de integração.

Use **especialmente** quando: sob pressão de tempo (emergências tornam o chute tentador), "só um fix rápido" parece óbvio, você já tentou várias correções, o fix anterior não funcionou, ou você não entende totalmente o problema.

Não pule quando: o bug "parece simples" (bugs simples também têm causa raiz) ou alguém quer "agora" (sistemático é mais rápido que ficar tateando).

---

## Steps — As Quatro Fases

Você DEVE completar cada fase antes de avançar para a próxima.

### Fase 1 — Investigação da Causa Raiz (antes de QUALQUER fix)

1. **Leia as mensagens de erro com atenção.** Não pule erros nem warnings — frequentemente contêm a solução exata. Leia o stack trace completo; anote linhas, paths, códigos de erro.
2. **Reproduza de forma consistente.** Consegue disparar de forma confiável? Quais os passos exatos? Acontece sempre? Se não reproduz → colete mais dados, não chute.
3. **Verifique mudanças recentes.** `git diff`, commits recentes, novas dependências, mudanças de config, diferenças de ambiente.
4. **Colete evidência em sistemas multi-componente** (CI → build → signing; API → service → database). Antes de propor fixes, adicione instrumentação de diagnóstico em CADA fronteira de componente: logue o que entra, o que sai, verifique propagação de env/config e o estado em cada camada. Rode uma vez para evidenciar ONDE quebra; só então investigue aquele componente específico.
5. **Rastreie o fluxo de dados.** Quando o erro está fundo no call stack: onde o valor ruim se origina? Quem chamou isso com o valor ruim? Suba até a fonte. Corrija na fonte, não no sintoma.

### Fase 2 — Análise de Padrão

1. **Ache exemplos que funcionam** — código similar que funciona no mesmo codebase.
2. **Compare com referências** — se está implementando um padrão, leia a implementação de referência COMPLETAMENTE, linha por linha. Não passe os olhos.
3. **Identifique as diferenças** entre o que funciona e o que está quebrado. Liste todas, por menores que pareçam. Não assuma "isso não pode importar".
4. **Entenda as dependências** — de que outros componentes/config/ambiente isso precisa? Que premissas assume?

### Fase 3 — Hipótese e Teste (método científico)

1. **Forme UMA hipótese.** Enuncie claramente: "Acho que X é a causa raiz porque Y." Escreva. Seja específico.
2. **Teste minimamente.** Faça a MENOR mudança possível para testar a hipótese. Uma variável por vez. Não conserte várias coisas de uma vez.
3. **Verifique antes de continuar.** Funcionou? → Fase 4. Não funcionou? → forme uma NOVA hipótese. NÃO empilhe mais fixes em cima.
4. **Quando não souber,** diga "não entendo X". Não finja. Peça ajuda ou pesquise mais.

### Fase 4 — Implementação (corrija a causa raiz, não o sintoma)

1. **Crie um teste que falha** reproduzindo o bug — a reprodução mais simples possível. OBRIGATÓRIO ter antes de corrigir. Use a skill `test-driven-development`.
2. **Implemente UM único fix** que ataca a causa raiz identificada. Uma mudança por vez. Nada de melhorias "já que estou aqui" nem refactoring agrupado.
3. **Verifique o fix.** O teste passa agora? Nenhum outro teste quebrou? O problema realmente foi resolvido?
4. **Se o fix não funcionar:** PARE. Conte quantos fixes você já tentou. Se < 3, volte à Fase 1 e reanalise com a nova informação. **Se ≥ 3, PARE e questione a arquitetura** — não tente o fix nº 4 sem discussão.
5. **Se 3+ fixes falharam — questione a arquitetura.** Padrões de problema arquitetural: cada fix revela novo estado compartilhado/acoplamento em outro lugar; fixes exigem "refactoring massivo"; cada fix cria sintomas novos em outro ponto. Pergunte: este padrão é fundamentalmente sólido? Estamos mantendo por pura inércia? Discuta com o humano antes de mais tentativas. Isto NÃO é hipótese falha — é arquitetura errada.

---

## Padrões

- Um sinal do humano de que você está errando: "Isso não está acontecendo?" (assumiu sem verificar), "Pare de chutar" (propôs fix sem entender), "Ultrathink isso" (questione os fundamentos), "Estamos travados?" (sua abordagem não está funcionando). Ao ver isto: volte à Fase 1.
- Racionalizações comuns e a realidade: "é simples, não preciso do processo" → bugs simples têm causa raiz; "emergência, sem tempo" → sistemático é mais rápido que tatear; "vários fixes de uma vez economizam tempo" → não dá pra isolar o que funcionou e causa novos bugs; "vejo o problema, deixa eu corrigir" → ver o sintoma ≠ entender a causa raiz.
- Se a investigação revelar que o problema é genuinamente ambiental/temporal/externo: documente o que investigou, implemente tratamento adequado (retry, timeout, mensagem de erro), adicione monitoramento. Mas 95% dos casos de "sem causa raiz" são investigação incompleta.

---

## Definition of Done

- [ ] Fase 1 completa: causa raiz entendida (o QUÊ e o PORQUÊ), não só o sintoma.
- [ ] Diferenças entre código funcionando e quebrado identificadas (Fase 2).
- [ ] Hipótese única formada e testada com a menor mudança possível (Fase 3).
- [ ] Teste que falha criado reproduzindo o bug, antes do fix.
- [ ] Um único fix aplicado, atacando a causa raiz; nada agrupado.
- [ ] Teste passa; nenhum outro teste quebrou; problema realmente resolvido.
- [ ] Se 3+ fixes falharam: arquitetura questionada e discutida, não fix nº 4 às cegas.

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/systematic-debugging/SKILL.md`.
- Skills relacionadas: `test-driven-development` (criar o teste que falha na Fase 4) e `verification-before-completion` (verificar que o fix funcionou antes de declarar sucesso).
- Impacto observado: abordagem sistemática 15-30 min/fix vs. 2-3h tateando; taxa de acerto de primeira ~95% vs. ~40%.
