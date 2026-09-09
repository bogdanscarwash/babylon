---
name: babylon-reconcile
description: Reconcile Babylon worktrees, origin/dev, Linear, GitHub, and durable task state before choosing or recovering work. Use for choosing actual next work from current repository and tracker state, portfolio audit, lost work, or unknown prior-session state. Do not use for a named bounded implementation.
---

<!-- vale off -->
<!-- Machine instructions preserve exact tracker and repository terms. -->

# Babylon reconciliation

Produce a current evidence map before recommending or mutating work.

1. Inspect the selected checkout, branches, worktrees, stashes, snapshots, and
   uncommitted state. A missing directory does not prove lost work.
2. Fetch and record the exact `origin/dev` identity. Use a clean temporary
   worktree when the current checkout is stale, detached, or dirty.
3. Read Linear for canonical scope, status, priority, dependencies, horizon,
   milestones, and ownership.
4. Read GitHub for code, PRs, reviews, checks, and historical evidence.
5. Reconcile durable task logs or memory only after live repository and tracker
   evidence. Label stale or conflicting observations.
6. Identify active, completed, blocked, recoverable, and ambiguous lanes.

Do not delete, merge, close, or reassign anything during a read-only audit.
Preserve ambiguous branches and worktrees until ancestry and ownership are
proved. If a connector is unavailable, report that gap instead of fabricating
tracker state.

Return a compact conflict list, the evidence for each conclusion, and the
smallest safe next action.

<!-- vale on -->
