---
name: babylon-lane
description: Implement, fix, resume coding, or finish a named Babylon PER issue through tested code and PR evidence. Use when the user names a bounded PER task. Do not use for lost-work recovery, read-only explanation, work selection, brainstorming, or an existing PR closeout.
---

<!-- vale off -->
<!-- Machine instructions preserve exact CLI and repository terms. -->

# Babylon lane

Execute the named issue without inventing adjacent scope.

1. Fetch the Linear issue and record its acceptance boundary and dependencies.
   If Linear is unavailable, report the gap and never fabricate tracker state.
2. Fetch `origin/dev`; inspect existing branches, worktrees, and PRs before
   creating anything. Resume recoverable work instead of duplicating it.
3. Use an isolated worktree from current `origin/dev` when the primary
   checkout is dirty or the change needs its own lane.
4. Translate acceptance into observable tests. Show RED, make the smallest
   implementation GREEN, then refactor.
5. Run the smallest relevant check after each change. Run the applicable
   repository gates before claiming completion; never overlap heavy gates.
6. Keep unrelated user changes untouched. Do not weaken requirements, baselines,
   or sentinels to obtain green output.
7. Update documentation only for verified behavior or a decision that must
   outlive the code.

Use `Part of PER-N` for a partial delivery and `Fixes PER-N` only when the
issue's final acceptance is satisfied. Do not commit directly to `dev` or
`main`.

Hand off exact branch, head SHA, tests, remaining risks, and PR state. Do not
start a generic planning or review workflow inside this skill.

<!-- vale on -->
