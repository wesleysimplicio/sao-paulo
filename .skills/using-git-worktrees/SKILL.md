---
name: using-git-worktrees
description: usar quando começar trabalho de feature que precisa de isolamento do workspace atual ou antes de executar um plano de implementação — garante um workspace isolado via ferramenta nativa ou fallback com git worktree
---

# Skill: `using-git-worktrees`

Garante que o trabalho aconteça em um workspace isolado. Prefira a ferramenta de worktree nativa da sua plataforma. Use `git worktree` manual apenas como fallback, quando não houver ferramenta nativa.

**Princípio central:** detecte isolamento existente primeiro → use ferramenta nativa → só então caia no fallback git. Nunca brigue com o harness.

---

## Trigger

- Ao iniciar trabalho de feature que precisa de isolamento do branch atual.
- Antes de executar um plano de implementação.
- Quando o usuário pedir "cria um worktree", "trabalha isolado", "não mexe no meu branch".
- Sempre que precisar proteger o branch atual de mudanças.

---

## Steps

1. **Detecte isolamento existente (Step 0).** Antes de criar qualquer coisa, verifique se você já está num workspace isolado:
   ```bash
   GIT_DIR=$(cd "$(git rev-parse --git-dir)" 2>/dev/null && pwd -P)
   GIT_COMMON=$(cd "$(git rev-parse --git-common-dir)" 2>/dev/null && pwd -P)
   BRANCH=$(git branch --show-current)
   ```
2. **Cheque o guard de submódulo.** `GIT_DIR != GIT_COMMON` também é verdadeiro dentro de submódulos. Confirme que não está num submódulo antes de concluir "já estou num worktree":
   ```bash
   # Se retornar um path, você está num submódulo — trate como repo normal
   git rev-parse --show-superproject-working-tree 2>/dev/null
   ```
3. **Se `GIT_DIR != GIT_COMMON` (e não é submódulo):** você já está num linked worktree. Pule para o Step de setup (passo 7). NÃO crie outro worktree. Reporte o branch (ou avise se for detached HEAD, gerenciado externamente).
4. **Se `GIT_DIR == GIT_COMMON` (ou é submódulo):** você está num checkout normal. Se o usuário ainda não declarou preferência, peça consentimento: *"Quer que eu configure um worktree isolado? Isso protege seu branch atual de mudanças."* Se recusar, trabalhe no lugar e pule para o passo 7.
5. **Crie o workspace — prefira ferramenta nativa (Step 1a).** Se você tem uma ferramenta tipo `EnterWorktree`, `WorktreeCreate`, comando `/worktree` ou flag `--worktree`, use-a e pule para o passo 7. Ferramentas nativas cuidam de diretório, criação de branch e cleanup automaticamente. Usar `git worktree add` com uma ferramenta nativa disponível cria estado fantasma que o harness não enxerga.
6. **Fallback git worktree (Step 1b) — só se NÃO houver ferramenta nativa.** Escolha o diretório nesta ordem de prioridade (preferência explícita do usuário sempre vence):
   1. Preferência declarada pelo usuário.
   2. Diretório project-local existente (`.worktrees` vence `worktrees`):
      ```bash
      ls -d .worktrees 2>/dev/null     # Preferido (oculto)
      ls -d worktrees 2>/dev/null      # Alternativa
      ```
   3. Diretório global legado: `~/.config/superpowers/worktrees/$project`.
   4. Default: `.worktrees/` na raiz do projeto.

   **Verificação de segurança (project-local):** confirme que o diretório é ignorado ANTES de criar. Se não for, adicione ao `.gitignore` e comite a mudança:
   ```bash
   git check-ignore -q .worktrees 2>/dev/null || git check-ignore -q worktrees 2>/dev/null
   ```
   Então crie o worktree:
   ```bash
   project=$(basename "$(git rev-parse --show-toplevel)")
   git worktree add "$path" -b "$BRANCH_NAME"
   cd "$path"
   ```
   **Fallback de sandbox:** se `git worktree add` falhar com erro de permissão, avise o usuário que o sandbox bloqueou e trabalhe no diretório atual.
7. **Rode o setup do projeto.** Auto-detecte e instale dependências:
   ```bash
   if [ -f package.json ]; then npm install; fi
   if [ -f Cargo.toml ]; then cargo build; fi
   if [ -f requirements.txt ]; then pip install -r requirements.txt; fi
   if [ -f pyproject.toml ]; then poetry install; fi
   if [ -f go.mod ]; then go mod download; fi
   ```
8. **Verifique o baseline limpo.** Rode a suíte de testes apropriada (`npm test` / `cargo test` / `pytest` / `go test ./...`). Se falhar, reporte e pergunte se deve prosseguir ou investigar. Se passar, reporte que está pronto.

---

## Padrões

- Ordem inviolável: **detectar isolamento → ferramenta nativa → fallback git**. Nunca pule direto para os comandos git do Step 1b.
- Nunca crie um worktree quando o Step 0 já detectou isolamento (evita worktree aninhado).
- Nunca use `git worktree add` quando há ferramenta nativa disponível — este é o erro nº 1.
- Para diretórios project-local, sempre verifique com `git check-ignore` antes de criar (evita comitar o conteúdo do worktree).
- Prioridade de diretório: existente > global legado > preferência declarada > default `.worktrees/`.
- Diretórios globais (`~/.config/superpowers/worktrees/`) não precisam de verificação de ignore.

---

## Definition of Done

- [ ] Step 0 (detecção de isolamento) executado antes de criar qualquer coisa.
- [ ] Guard de submódulo checado antes de concluir "já estou num worktree".
- [ ] Ferramenta nativa preferida ao fallback git quando disponível.
- [ ] Diretório project-local verificado como ignorado (`git check-ignore`) antes da criação.
- [ ] Setup de dependências auto-detectado e executado.
- [ ] Baseline de testes verificado limpo (ou falhas reportadas com pedido de decisão).
- [ ] Report final: path do worktree + status dos testes + feature a implementar.

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/using-git-worktrees/SKILL.md`.
- Gotcha: rodar `git worktree remove` de dentro do próprio worktree falha silenciosamente — sempre `cd` para a raiz do repo principal antes.
- Complementa a skill `finishing-a-development-branch`, que cuida do cleanup do worktree ao concluir.
