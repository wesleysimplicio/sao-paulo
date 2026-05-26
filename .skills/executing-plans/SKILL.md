---
name: executing-plans
description: ativar quando houver um plano de implementação escrito para executar, task a task e em ordem, verificando cada step com checkpoints de review
---

# Skill: `executing-plans`

Carregue o plano, revise-o criticamente, execute todas as tasks na ordem e reporte ao concluir. Cada task tem steps bite-sized — siga-os exatamente e não pule verificações.

> **Anuncie no início:** "Estou usando a skill executing-plans para implementar este plano."

---

## Trigger

- Quando houver um plano de implementação escrito (ex.: `docs/plans/...`) pronto para executar.
- Quando o usuário escolher "execução inline" no handoff da skill `writing-plans`.

---

## Steps

1. **Carregue e revise o plano** — leia o arquivo do plano e revise criticamente, identificando dúvidas ou preocupações. Se houver, levante com o usuário ANTES de começar. Se não, crie o TodoWrite e prossiga.
2. **Execute as tasks em ordem.** Para cada task: marque como `in_progress`; siga cada step exatamente (o plano tem steps bite-sized); rode as verificações como especificado; marque como `completed`.
3. **Volte ao Step 1 (review)** quando o usuário atualizar o plano com base no seu feedback ou quando a abordagem fundamental precisar ser repensada.
4. **Conclua o desenvolvimento** após todas as tasks completas e verificadas — verifique os testes, apresente as opções de finalização e execute a escolha (skill de finalização de branch, se disponível).

---

## Padrões

- **Faça as tasks em ordem** e siga os steps do plano exatamente — não improvise.
- **Não pule verificações.** Cada step verificável é rodado e confirmado antes de marcar como concluído.
- **Referencie skills** quando o plano mandar.
- **PARE de executar imediatamente** quando: bater num blocker (dependência faltando, teste falha, instrução obscura); o plano tiver lacunas críticas que impeçam começar; você não entender uma instrução; ou uma verificação falhar repetidamente.
- **Peça esclarecimento em vez de adivinhar.** Não force a passagem por blockers — pare e pergunte.
- **Nunca inicie implementação em main/master** sem consentimento explícito do usuário.

---

## Definition of Done

- [ ] Plano lido e revisado criticamente antes de iniciar; preocupações levantadas com o usuário.
- [ ] TodoWrite criado a partir das tasks/steps do plano.
- [ ] Todas as tasks executadas em ordem, seguindo os steps exatamente.
- [ ] Todas as verificações do plano rodadas e passando (nenhuma pulada).
- [ ] Blockers tratados parando e perguntando, sem adivinhar.
- [ ] Desenvolvimento finalizado (testes verificados, opções de finalização apresentadas).

---

## Notas

- Adaptado de **superpowers** (https://github.com/obra/superpowers) — Jesse Vincent, MIT License.
- Skill original: `skills/executing-plans/SKILL.md`.
- Superpowers funciona melhor com acesso a subagentes. Em plataforma com suporte a subagentes, prefira a abordagem subagent-driven (um subagente novo por task com review em dois estágios) em vez desta skill de execução inline.
- Skills de workflow relacionadas: criação de worktree isolado antes de começar e `writing-plans` (que produz o plano que esta skill executa).
