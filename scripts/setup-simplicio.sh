#!/usr/bin/env bash
# Setup for simplicio-cli (https://github.com/wesleysimplicio/simplicio-cli).
# Installs the CLI and validates the provider, idempotently.
# Safe to run on every session start; never aborts the session.
set -uo pipefail

log() { printf '[simplicio-setup] %s\n' "$1" >&2; }

PY=""
for c in python3 python; do
  if command -v "$c" >/dev/null 2>&1; then PY="$c"; break; fi
done
if [ -z "$PY" ]; then
  log "Python 3.9+ not found; install Python to use simplicio-cli."
  exit 0
fi

if command -v simplicio >/dev/null 2>&1; then
  log "already installed: $(simplicio --version 2>/dev/null || echo present)"
else
  log "installing simplicio-cli..."
  if command -v pipx >/dev/null 2>&1; then
    pipx install simplicio-cli >&2 || log "pipx install failed"
  else
    "$PY" -m pip install --upgrade simplicio-cli >&2 \
      || "$PY" -m pip install --user --upgrade simplicio-cli >&2 \
      || "$PY" -m pip install --break-system-packages --upgrade simplicio-cli >&2 \
      || log "pip install failed (check network / PyPI availability)"
  fi
fi

if ! command -v simplicio >/dev/null 2>&1; then
  log "simplicio not on PATH after install; add the pip user bin dir to PATH."
  exit 0
fi

if [ -n "${ANTHROPIC_API_KEY:-}" ] || [ -n "${OPENAI_API_KEY:-}" ] \
   || [ -n "${DEEPSEEK_API_KEY:-}" ] || [ -n "${SIMPLICIO_BASE_URL:-}" ]; then
  log "validating provider with 'simplicio smoke'..."
  simplicio smoke >&2 || log "smoke failed (check provider env vars)"
else
  log "no provider key set; skipping smoke. Set ANTHROPIC_API_KEY (or another provider), then run: simplicio smoke"
fi
