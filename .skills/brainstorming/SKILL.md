---
name: brainstorming
description: ativar OBRIGATORIAMENTE antes de qualquer trabalho criativo — criar features, construir componentes, adicionar funcionalidade ou mudar comportamento — para explorar intenção, requisitos e design antes de implementar
---

# Skill: `brainstorming`

Transforme ideias em designs e specs completos por meio de diálogo colaborativo. Entenda o contexto do projeto, faça perguntas uma de cada vez para refinar a ideia e, quando souber o que será construído, apresente o design e obtenha a aprovação do usuário.

---

## HARD-GATE

> NÃO invoque nenhuma skill de implementação, NÃO escreva código, NÃO faça scaffold de projeto e NÃO tome nenhuma ação de implementação enquanto não tiver apresentado um design **e** o usuário não tiver aprovado. Isso vale para TODO projeto, independente da simplicidade percebida.

### Antipadrão: "Isso é simples demais pra precisar de design"

Todo projeto passa por este processo. Uma todo list, um utilitário de uma função, uma mudança de config — todos. Projetos "simples" são exatamente onde suposições não examinadas causam mais retrabalho. O design pode ser curto (algumas frases para projetos realmente simples), mas você DEVE apresentá-lo e obter aprovação.

---

## Trigger

- Quando o usuário pedir "vamos construir X", "cria a feature Y", "adiciona Z", "muda o comportamento de W".
- Antes de entrar em plan mode, se ainda não houve brainstorming.
- Antes de qualquer ação criativa ou de implementação — esta skill vem primeiro.

---

## Steps

Crie uma task de TodoWrite para cada item e complete-os em ordem:

1. **Explore o contexto do projeto** — leia arquivos, docs, commits recentes.
2. **Avalie o escopo**: se o pedido descreve múltiplos subsistemas independentes (ex.: "plataforma com chat, storage, billing e analytics"), sinalize de imediato e ajude a decompor em sub-projetos antes de refinar detalhes. Cada sub-projeto tem seu próprio ciclo spec → plano → implementação.
3. **Ofereça o companion visual** (se as perguntas envolverem conteúdo visual) — em mensagem própria, sem nenhum outro conteúdo. Aguarde a resposta.
4. **Faça perguntas de esclarecimento** — uma de cada vez, foco em propósito, restrições e critérios de sucesso. Prefira múltipla escolha quando possível.
5. **Proponha 2-3 abordagens** com trade-offs; lidere com sua recomendação e o porquê.
6. **Apresente o design** em sections dimensionadas à complexidade (poucas frases se direto; até 200-300 palavras se houver nuance). Cubra arquitetura, componentes, fluxo de dados, tratamento de erros e testes. Pergunte após cada section se está certo e obtenha aprovação.
7. **Escreva o design doc** em `docs/specs/YYYY-MM-DD-<topico>-design.md` (preferências do usuário sobre o local prevalecem) e commit.
8. **Self-review da spec**: varredura de placeholders (TBD/TODO), consistência interna, escopo, ambiguidade. Corrija inline, sem re-review.
9. **Peça ao usuário para revisar a spec escrita** antes de prosseguir. Se pedirem mudanças, faça e re-rode a self-review.
10. **Transição para implementação** — invoque a skill `writing-plans`. Esta é a ÚNICA skill a invocar após o brainstorming.

---

## Padrões

- **Uma pergunta por mensagem** — não sobrecarregue com várias.
- **Múltipla escolha preferida** — mais fácil de responder que aberta.
- **YAGNI sem dó** — remova features desnecessárias de todos os designs.
- **Explore alternativas** — sempre proponha 2-3 abordagens antes de fechar.
- **Validação incremental** — apresente, aprove, então avance.
- **Seja flexível** — volte e esclareça quando algo não fizer sentido.
- **Design para isolamento**: quebre o sistema em unidades pequenas, cada uma com um propósito claro, interfaces bem definidas e testáveis isoladamente. Para cada unidade você deve saber: o que faz, como se usa, do que depende.
- **Em codebases existentes**: explore a estrutura atual e siga os padrões. Inclua melhorias pontuais só onde elas servem ao objetivo atual; não proponha refactor não relacionado.
- O estado terminal é invocar `writing-plans`. NÃO invoque nenhuma outra skill de implementação após o brainstorming.

---

## Definition of Done

- [ ] Contexto do projeto explorado antes das perguntas.
- [ ] Perguntas feitas uma de cada vez; 2-3 abordagens propostas com trade-offs.
- [ ] Design apresentado em sections e aprovado pelo usuário (HARD-GATE respeitado: nenhum código antes da aprovação).
- [ ] Design doc escrito em `docs/specs/YYYY-MM-DD-<topico>-design.md` e commitado.
- [ ] Self-review da spec feita (sem placeholders, sem contradições, escopo OK, sem ambiguidade).
- [ ] Usuário revisou e aprovou a spec escrita.
- [ ] Skill `writing-plans` invocada como passo seguinte (e nenhuma outra skill de implementação).

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/brainstorming/SKILL.md`.
- Companion visual é uma ferramenta, não um modo: aceitar significa que ele fica disponível para perguntas que se beneficiem de tratamento visual, não que toda pergunta passe pelo navegador. Por pergunta, use o navegador quando o conteúdo É visual (mockups, layouts, diagramas) e o terminal quando é texto (requisitos, escolhas conceituais, trade-offs).
