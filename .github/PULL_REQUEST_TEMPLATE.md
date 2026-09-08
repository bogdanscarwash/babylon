<!-- Vale: this template preserves literal Git, GitHub, Linear, and CI terms. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->

## Summary

<!-- Describe the change and its player-facing or engineering purpose. -->

## Linear delivery

<!-- Every PR must select exactly one disposition. Replace N with the issue number. -->

- [ ] `Part of PER-N` — partial delivery. Keep the Linear issue open.
- [ ] `Fixes PER-N` — final accepted delivery. Close the Linear issue after merge.

Linear issue: <!-- Link the canonical Linear issue. -->

## Review evidence

Base branch: <!-- Normally dev. -->

Exact reviewed head SHA: <!-- Use the full 40-character commit SHA. -->

- [ ] The base branch is `dev`, or the Director approved `main`.
  - Release PR from `dev`.
  - Critical hotfix from `fix/*`, with a mandatory backport PR to `dev`.
- [ ] For a `main` target, current `origin/main` is an ancestor of the exact
      reviewed head before `main.yml` passed on that head.
- [ ] For a `main` target, this PR produced the complete combined manifest.
- [ ] Blocking checks completed successfully for the exact reviewed head
      SHA and base branch above. Verified asynchronous CodeQL scans may remain
      active on `dev`; known findings still block. A `main` target requires
      successful exact-head CodeQL and zero open PR findings.
- [ ] Copilot evidence is advisory when absent, stale, incomplete, API-failed,
      identity-mismatched, or without a reply.
- [ ] No unresolved review thread remains. Any unresolved review thread blocks
      the merge, regardless of its author.

## Behavioral-contract disposition

<!-- Select one disposition and link the evidence or give the explanation. -->

- [ ] Changed behavior: I added or updated a durable behavioral contract.
- [ ] No behavior change: the current behavioral contracts are enough.

Disposition and evidence:

## Baseline disposition

<!-- Select one disposition. Never bless a baseline to hide a fault. -->

- [ ] No governed baseline changed.
- [ ] A governed baseline changed intentionally. I used
      `tools/generate_ceremony_message.py` and included the required ceremony
      record and `Baselines: blessed(<slug>)` trailer.

## Merge

- [ ] Merge only with `mise run pr:merge -- N`. Use this PR number for `N`.
- [ ] For a `main` target, only the Director uses
      `mise run pr:merge -- N --director-main`.
- [ ] Preserve the source branch by default. Delete it only after an explicit
      owner decision and a check for dependent work.

Do not run `gh pr merge` directly in any form.

## Questions for reviewers

<!-- Identify unresolved questions. Leave blank when there are none. -->

______________________________________________________________________

**New contributor?** Read [CONTRIBUTORS.md](../CONTRIBUTORS.md) before you open
the PR.

<!-- vale ste.UnapprovedWords = YES -->
<!-- vale Vale.Spelling = YES -->
