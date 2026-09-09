---
name: babylon-pr-cycle
description: Qualify, repair, monitor, and merge an existing Babylon pull request with exact-head CI, review-thread, and Linear evidence. Use for finish, babysit, fix CI, review, or merge PR N. Do not use before a PR exists.
---

<!-- vale off -->
<!-- Machine instructions preserve exact PR and CI terms. -->

# Babylon PR cycle

Keep every conclusion pinned to the current PR head.

1. Fetch the PR, base, exact head SHA, mergeability, linked PER issue, checks,
   reviews, and unresolved threads.
2. Separate required evidence from advisory evidence. Copilot absence or delay
   is advisory; an unresolved thread remains blocking.
3. Reproduce each real defect with a RED behavioral test before changing code.
   Repair only the causal scope, then rerun the smallest failing check.
4. After every pushed change, discard stale conclusions and requalify the new
   exact head. A watcher finishing is not evidence that a check passed.
5. Do not merge while required CI, exact-head identity, mergeability, delivery
   disposition, or review-thread evidence is unresolved.
6. Use only `mise run pr:merge -- N` after all hard boundaries are green.
   Never use direct or automatic GitHub merge commands.
7. Verify the resulting merge SHA and Linear status. Correct accidental
   completion when a partial delivery used the wrong closing disposition.

If the cycle is waiting on external state, monitor it without generating new
plans or loading unrelated governance. Report exact blockers and their evidence.

<!-- vale on -->
