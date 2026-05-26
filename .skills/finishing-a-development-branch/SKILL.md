---
name: finishing-a-development-branch
description: usar quando a implementação está completa, todos os testes passam e você precisa decidir como integrar o trabalho — guia a conclusão apresentando opções estruturadas de merge, PR ou cleanup
---

# Skill: `finishing-a-development-branch`

Guia a conclusão de um trabalho de desenvolvimento apresentando opções claras e executando o fluxo escolhido.

**Princípio central:** verificar testes → detectar ambiente → apresentar opções → executar escolha → fazer cleanup.

---

## Trigger

- Quando a implementação está completa e os testes passam.
- Quando o usuário pedir "finaliza o branch", "abre o PR", "faz merge", "termina isso".
- Ao concluir a execução de um plano (ex.: ao final de `subagent-driven-development`).

---

## Steps

1. **Verifique os testes primeiro.** Rode a suíte do projeto (`npm test` / `cargo test` / `pytest` / `go test ./...`). Se falharem, **pare**: mostre as falhas e avise que não dá para fazer merge/PR até passarem. Não avance.
2. **Detecte o ambiente.** Determine o estado do workspace, que define qual menu mostrar e como funciona o cleanup:
   ```bash
   GIT_DIR=$(cd "$(git rev-parse --git-dir)" 2>/dev/null && pwd -P)
   GIT_COMMON=$(cd "$(git rev-parse --git-common-dir)" 2>/dev/null && pwd -P)
   ```
   | Estado | Menu | Cleanup |
   |--------|------|---------|
   | `GIT_DIR == GIT_COMMON` (repo normal) | 4 opções padrão | Sem worktree para limpar |
   | `GIT_DIR != GIT_COMMON`, branch nomeado | 4 opções padrão | Baseado em proveniência (passo 6) |
   | `GIT_DIR != GIT_COMMON`, detached HEAD | 3 opções (sem merge) | Sem cleanup (gerenciado externamente) |
3. **Determine o branch base.** Tente `git merge-base HEAD main` ou `git merge-base HEAD master`. Em caso de dúvida, pergunte: *"Esse branch saiu de main, correto?"*
4. **Apresente as opções (sem explicação extra, conciso).**

   Repo normal e worktree com branch nomeado — exatamente estas 4 opções:
   ```
   Implementação completa. O que você quer fazer?

   1. Merge de volta para <base-branch> localmente
   2. Push e abrir um Pull Request
   3. Manter o branch como está (eu cuido depois)
   4. Descartar este trabalho

   Qual opção?
   ```
   Detached HEAD — exatamente estas 3 opções:
   ```
   Implementação completa. Você está em detached HEAD (workspace gerenciado externamente).

   1. Push como branch novo e abrir um Pull Request
   2. Manter como está (eu cuido depois)
   3. Descartar este trabalho

   Qual opção?
   ```
5. **Execute a escolha.**
   - **Opção 1 — Merge local:** vá para a raiz do repo principal, faça o merge e só então limpe.
     ```bash
     MAIN_ROOT=$(git -C "$(git rev-parse --git-common-dir)/.." rev-parse --show-toplevel)
     cd "$MAIN_ROOT"
     git checkout <base-branch>
     git pull
     git merge <feature-branch>
     <comando de teste>   # verifique os testes no resultado mergeado
     ```
     Só depois do merge bem-sucedido: cleanup do worktree (passo 6) e então `git branch -d <feature-branch>`.
   - **Opção 2 — Push e PR:**
     ```bash
     git push -u origin <feature-branch>
     gh pr create --title "<title>" --body "$(cat <<'EOF'
     ## Summary
     <2-3 bullets do que mudou>

     ## Test Plan
     - [ ] <passos de verificação>
     EOF
     )"
     ```
     **NÃO limpe o worktree** — o usuário precisa dele vivo para iterar no feedback do PR.
   - **Opção 3 — Manter:** reporte "Mantendo branch <name>. Worktree preservado em <path>." Sem cleanup.
   - **Opção 4 — Descartar:** **confirme primeiro**, listando branch, commits e worktree a serem apagados, e exija que o usuário digite `discard`. Só então: `cd` para a raiz do repo, cleanup do worktree (passo 6), e `git branch -D <feature-branch>`.
6. **Cleanup do workspace (só Opções 1 e 4; 2 e 3 sempre preservam).**
   ```bash
   WORKTREE_PATH=$(git rev-parse --show-toplevel)
   ```
   - Se `GIT_DIR == GIT_COMMON`: repo normal, nada a limpar.
   - Se o path está sob `.worktrees/`, `worktrees/` ou `~/.config/superpowers/worktrees/`: nós criamos, nós limpamos:
     ```bash
     MAIN_ROOT=$(git -C "$(git rev-parse --git-common-dir)/.." rev-parse --show-toplevel)
     cd "$MAIN_ROOT"
     git worktree remove "$WORKTREE_PATH"
     git worktree prune
     ```
   - Caso contrário: o harness é dono do workspace. **NÃO remova.** Use a ferramenta de saída da plataforma, se houver.

---

## Padrões

- **Nunca prossiga com testes falhando** — sempre verifique antes de oferecer opções.
- Apresente **exatamente** 4 opções (ou 3 no detached HEAD). Nada de perguntas abertas tipo "o que faço agora?".
- Faça merge **antes** de remover o worktree e de deletar o branch (`git branch -d` falha se o worktree ainda referencia o branch).
- Sempre `cd` para a raiz do repo principal antes de `git worktree remove` (rodar de dentro do worktree falha silenciosamente).
- **Nunca faça force-push** sem pedido explícito — e nunca em branches compartilhados.
- Só limpe worktrees que **você criou** (checagem de proveniência). Worktrees do harness não são seus.
- Exija confirmação digitada (`discard`) antes de apagar trabalho.

---

## Definition of Done

- [ ] Testes verificados como passando antes de qualquer opção.
- [ ] Ambiente detectado (`GIT_DIR` vs `GIT_COMMON`) e menu correto apresentado.
- [ ] Branch base determinado ou confirmado com o usuário.
- [ ] Escolha do usuário executada conforme o fluxo da opção.
- [ ] Para merge: testes verificados no resultado mergeado antes do cleanup.
- [ ] Cleanup do worktree feito só nas Opções 1 e 4, e só para worktrees próprios.
- [ ] `git worktree prune` rodado após remoção.
- [ ] Nenhum force-push feito sem pedido explícito.

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/finishing-a-development-branch/SKILL.md`.
- Tabela de referência rápida das opções: 1) merge+delete branch; 2) push+PR (mantém worktree); 3) mantém tudo; 4) descarta (force-delete, mantém worktree antes do cleanup final).
- Complementa `using-git-worktrees`, que cria o workspace isolado no início.
