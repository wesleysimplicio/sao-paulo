# Third-party skills — NOTICE

The following skill directories under `.skills/` are vendored verbatim from
the upstream HyperFrames project and are distributed under the Apache
License, Version 2.0:

- `hyperframes/`
- `hyperframes-cli/`
- `hyperframes-media/`
- `hyperframes-registry/`
- `gsap/`
- `animejs/`
- `css-animations/`
- `lottie/`
- `three/`
- `waapi/`
- `tailwind/`
- `typegpu/`
- `contribute-catalog/`
- `remotion-to-hyperframes/`
- `website-to-hyperframes/`

Upstream source: https://github.com/wesleysimplicio/hyperframes

License text: see `.skills/UPSTREAM-LICENSE` (Apache 2.0).

Modifications: none. Files are copied as-is from `skills/<name>/SKILL.md`
in the upstream repository at the time of import. To refresh, run:

```bash
for s in hyperframes hyperframes-cli hyperframes-media hyperframes-registry \
         gsap animejs css-animations lottie three waapi tailwind typegpu \
         contribute-catalog remotion-to-hyperframes website-to-hyperframes; do
  curl -sSf \
    "https://raw.githubusercontent.com/wesleysimplicio/hyperframes/main/skills/${s}/SKILL.md" \
    -o ".skills/${s}/SKILL.md"
done
```

## Adapted skills — superpowers (MIT)

The following skill directories under `.skills/` are **adaptations** (not
verbatim copies) of skills from the superpowers project. They were rewritten
into this project's Brazilian-Portuguese skill format (Trigger / Steps /
Padrões / Definition of Done / Notas) while preserving the upstream
methodology:

- `using-superpowers/`
- `brainstorming/`
- `writing-plans/`
- `executing-plans/`
- `test-driven-development/`
- `systematic-debugging/`
- `verification-before-completion/`
- `using-git-worktrees/`
- `finishing-a-development-branch/`
- `dispatching-parallel-agents/`
- `subagent-driven-development/`
- `requesting-code-review/`
- `receiving-code-review/`
- `writing-skills/`

Upstream source: https://github.com/obra/superpowers
Copyright (c) 2025 Jesse Vincent. Licensed under the MIT License.

Each adapted `SKILL.md` cites the original upstream skill path in its `## Notas`
section. To compare against upstream, fetch:

```bash
for s in using-superpowers brainstorming writing-plans executing-plans \
         test-driven-development systematic-debugging verification-before-completion \
         using-git-worktrees finishing-a-development-branch dispatching-parallel-agents \
         subagent-driven-development requesting-code-review receiving-code-review writing-skills; do
  curl -sSf \
    "https://raw.githubusercontent.com/obra/superpowers/main/skills/${s}/SKILL.md" \
    -o "/tmp/${s}.upstream.md"
done
```

## Project-original skills

Local skills (`_template/`, `caveman/`, `ralph-loop/`, `everything-claude-code/`,
`playwright-e2e/`, `conventional-commits/`, `rtk-cli/`) are project-original
and not covered by this notice.
