---
name: test-driven-development
description: ative ao implementar qualquer feature ou bugfix, antes de escrever código de produção — teste primeiro, veja falhar, só então implemente
---

# Skill: `test-driven-development`

Escreva o teste primeiro. Veja ele falhar. Escreva o código mínimo para passar. **Princípio central:** se você não assistiu o teste falhar, você não sabe se ele testa a coisa certa.

> **Violar a letra das regras é violar o espírito das regras.** Não há "só dessa vez".

---

## Trigger

- Ao implementar qualquer feature nova.
- Ao corrigir qualquer bug (escreva um teste que reproduz o bug primeiro).
- Ao refatorar ou mudar comportamento existente.
- Quando o usuário pedir "implementa X", "corrige Y", "adiciona Z".
- **Exceções** (peça confirmação ao humano): protótipos descartáveis, código gerado, arquivos de configuração.

Pensou "pular TDD só dessa vez"? Pare. Isso é racionalização.

---

## A Lei de Ferro

```
NENHUM CÓDIGO DE PRODUÇÃO SEM UM TESTE QUE FALHA PRIMEIRO
```

Escreveu código antes do teste? Delete. Comece de novo. Sem exceções:
- Não guarde "como referência".
- Não "adapte" enquanto escreve os testes.
- Não olhe pra ele. Deletar significa deletar.

Reimplemente do zero a partir dos testes.

---

## Steps — Ciclo RED-GREEN-REFACTOR

1. **RED — escreva um teste que falha.** Um comportamento, nome claro, código real (sem mocks a menos que inevitável).
2. **Verifique o RED — assista falhar. OBRIGATÓRIO, nunca pule.** Rode o teste (ex.: `npm test path/to/test.test.ts`). Confirme que: o teste **falha** (não dá erro de typo), a mensagem de falha é a esperada, e falha porque a feature não existe.
   - Teste passou? Você está testando comportamento já existente. Conserte o teste.
   - Teste deu erro (não falha)? Conserte o erro e rode de novo até falhar corretamente.
3. **GREEN — escreva o código mínimo para passar.** Nada além do necessário. Sem features extras, sem over-engineering (YAGNI).
4. **Verifique o GREEN — assista passar. OBRIGATÓRIO.** Rode de novo. Confirme: o teste passa, os outros testes continuam passando, output limpo (sem erros nem warnings).
   - Teste falha? Conserte o **código**, não o teste.
   - Outros testes quebraram? Conserte agora.
5. **REFACTOR — limpe.** Só depois do verde: remova duplicação, melhore nomes, extraia helpers. Mantenha tudo verde, não adicione comportamento.
6. **Repita.** Próximo teste que falha, próxima feature.

---

## Padrões

- **Bom teste:** mínimo (um comportamento — "and" no nome? divida), nome descreve o comportamento, demonstra a API desejada com código real.
- **Ruim:** testar o mock em vez do código, nomes vagos (`test1`, "retry works"), mais de uma coisa por teste.
- Não adicione testes "depois". Teste escrito após o código passa de imediato e isso não prova nada — pode testar a coisa errada, a implementação em vez do comportamento, ou perder edge cases.
- Bug encontrado? Escreva um teste que falha reproduzindo-o, então siga o ciclo. Nunca corrija bug sem teste.
- **Quando travar:** teste complicado demais = design complicado demais (simplifique a interface). Precisa mockar tudo = código acoplado demais (use injeção de dependência).

### Exemplo: bugfix

```typescript
// RED
test('rejects empty email', async () => {
  const result = await submitForm({ email: '' });
  expect(result.error).toBe('Email required');
});
```
```bash
$ npm test
FAIL: expected 'Email required', got undefined   # verifique o RED
```
```typescript
// GREEN
function submitForm(data: FormData) {
  if (!data.email?.trim()) return { error: 'Email required' };
  // ...
}
```
```bash
$ npm test
PASS   # verifique o GREEN
```

---

## Red Flags — PARE e comece de novo

- Código antes do teste; teste depois da implementação.
- Teste passa de imediato; não sabe explicar por que falhou.
- "Já testei manualmente"; "testes depois alcançam o mesmo objetivo"; "é sobre o espírito, não o ritual".
- "Guardar como referência" ou "adaptar código existente".
- "Já gastei X horas, deletar é desperdício" (sunk cost — código não confiável é dívida técnica).
- "TDD é dogmático, estou sendo pragmático" (TDD **é** pragmático: acha bugs antes do commit, previne regressões, documenta comportamento).

Qualquer um destes significa: delete o código, comece de novo com TDD.

---

## Definition of Done

- [ ] Cada função/método novo tem um teste.
- [ ] Assisti cada teste **falhar** antes de implementar.
- [ ] Cada teste falhou pelo motivo esperado (feature ausente, não typo).
- [ ] Escrevi o código mínimo para passar cada teste.
- [ ] Todos os testes passam; output limpo (sem erros nem warnings).
- [ ] Testes usam código real (mocks só se inevitável).
- [ ] Edge cases e erros cobertos.

Não consegue marcar todos? Você pulou TDD. Comece de novo.

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/test-driven-development/SKILL.md`.
- Regra final: código de produção → existe um teste que falhou primeiro; caso contrário → não é TDD. Sem exceções sem permissão do humano.
- Integra com `systematic-debugging` (criar o teste que falha na Fase 4) e `verification-before-completion`.
