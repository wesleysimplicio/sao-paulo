#!/usr/bin/env bash
# PreToolUse guard (Edit|Write): coding in this repo must go through simplicio-cli,
# so the CLI must be set up before any edit. Exit 2 blocks and shows stderr to the agent.

# Escape hatch for environments without network or provider keys.
if [ "${SIMPLICIO_BYPASS:-}" = "1" ]; then
  exit 0
fi

if command -v simplicio >/dev/null 2>&1; then
  exit 0
fi

cat >&2 <<'EOF'
[simplicio] BLOCKED: simplicio-cli must be set up before editing code in this repo.

Using simplicio-cli for coding tasks is mandatory. Set it up first:

    bash scripts/setup-simplicio.sh

Then drive the change through the CLI, e.g.:

    simplicio task "<describe the change>" --target <file> --criteria "<acceptance>"

If this environment has no network or provider keys, bypass with SIMPLICIO_BYPASS=1.
EOF
exit 2
