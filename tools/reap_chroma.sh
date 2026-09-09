#!/usr/bin/env bash
# Reap leaked chroma-mcp servers. The claude-mem / ChromaDB MCP backend spawns one
# ~1 GB server per connection and never reaps them; on a long session they pile up
# (77 = 28 GB once left the dev box one test-spike short of a freeze). Safe to kill:
# the Chroma store is persistent on disk, so a fresh server just respawns on next use.
#
# Self-guard: ``chroma[-]mcp`` is a bracket regex that matches the literal
# ``chroma-mcp`` in the server command lines but NOT this pattern's own text, so the
# reaper can't SIGTERM its own shell (the exit-144 trap of a bare `pkill -f chroma-mcp`).
# Running from a file — the process command line is the path, not the script body —
# makes that guarantee airtight (an inline `mise run` block does not).
set -uo pipefail

# pgrep -fc always prints the count to stdout but exits non-zero when it is 0, so
# take its output and default an empty result to 0 (never chain `|| echo 0`, which
# would append a second 0 and corrupt the arithmetic below).
pattern='chroma[-]mcp'
before=$(pgrep -fc "$pattern" 2>/dev/null || true)
before=${before:-0}
if [ "$before" -gt 0 ]; then
  pkill -f "$pattern" 2>/dev/null || true
  sleep 2
fi
after=$(pgrep -fc "$pattern" 2>/dev/null || true)
after=${after:-0}
echo "chroma-mcp servers: ${before} -> ${after}  (~$((before - after)) GB reclaimed)"
