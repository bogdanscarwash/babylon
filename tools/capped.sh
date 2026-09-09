#!/usr/bin/env bash
# Run any command inside a transient systemd *user* scope with a HARD memory
# and CPU ceiling. If the command's cgroup exceeds MemoryMax it is OOM-killed
# *in isolation* — the desktop and the rest of the machine are untouched.
#
# Why this exists: Babylon's test suite (`mise run test:unit`) runs pytest-xdist
# with full-tree coverage instrumentation — ~1 GB per worker — and mutmut /
# parallel agent fan-outs can stack many of those at once. On a box with no
# earlyoom / systemd-oomd, that memory pressure thrashes swap and freezes the
# whole UI before the kernel OOM killer fires. A per-command cgroup cap makes
# that impossible: the runaway dies, the machine lives.
#
# MemorySwapMax=0 is load-bearing: it forbids the capped command from touching
# swap at all, so it can never *thrash* — it hits the ceiling and dies cleanly.
#
# Usage:   tools/capped.sh <command...>            (or: mise run cap -- <command...>)
# Tune:    CAP_MEM=8G CAP_CPU=600% tools/capped.sh <command...>
# Defaults: 12 GiB memory (of 31), 800% CPU (8 of 12 cores) — leaves headroom
# for the desktop no matter what the command does.
set -euo pipefail

CAP_MEM="${CAP_MEM:-12G}"
CAP_CPU="${CAP_CPU:-800%}"

if [ "$#" -eq 0 ]; then
  echo "usage: capped.sh <command...>   (env: CAP_MEM=$CAP_MEM CAP_CPU=$CAP_CPU)" >&2
  exit 2
fi

echo "[capped] MemoryMax=$CAP_MEM MemorySwapMax=0 CPUQuota=$CAP_CPU :: $*" >&2
exec systemd-run --user --scope --quiet \
  -p MemoryMax="$CAP_MEM" \
  -p MemorySwapMax=0 \
  -p CPUQuota="$CAP_CPU" \
  -- "$@"
