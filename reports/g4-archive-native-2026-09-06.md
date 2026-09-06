# Native Archive quiet-week and restart evidence

This receipt covers PER-320's visible quiet-week refresh and restart behavior.
It does not prove G4 comprehension or performance acceptance.

## Source and session

- Native binaries built from commit
  `1f8eb154cc5c226d4af0972add92b4e5ab9c6a1b` before the agent added the
  command-ID review regression.
- Standard Michigan campaign:
  `e89c3b3e-bf6f-4637-97c8-0666c80d9feb`.
- Dedicated QA database: `babylon_g4_ui_qa_b90d_20260904_v2`.
- County: Wayne, `26163`. Full observer. Actual window: `1366×768`.
- The Director explicitly requested restarting the application and testing it.
  The earlier user-operated session in `launch.log` is separate from this proof.
- Test client PID `1827492`, launcher PID `1826105`.
- Restart client PID `1890047`, launcher PID `1888638`.
- The input helper verified the client executable, window PID, and launcher parent.
- Both test launchers exited with status zero. No failed session state appeared.

## Observed sequence

1. Reopen the Standard campaign at week zero. Open Wayne's cited Archive.
2. Activate WORLD to release control focus without closing the Archive.
3. Advance once to week one. The held card moves from unavailable through pending publication to verified.
4. Keep Wayne selected and the Archive open throughout both advances.
5. At `11:32:41.234436Z`, the card installs week-one content, verified through week one.
6. Capture the visible card and the confined dossier CLI row.
7. Advance at `11:36:13.674993Z`, keeping the card open. The durable acknowledgement reaches week two at `11:36:23.664036Z`.
8. At `11:36:27.565657Z`, the card installs existing week-one content with verification through week two. The screenshot shows both facts.
9. Close the owned window through its normal window-manager close event.
10. Reopen the same campaign without advancing, and open Wayne's Archive.
11. The card shows week two, verification through two, and content from one.
12. Before and after restart, complete CLI output bytes match, including scope, citations, content, and freshness.

## Exact checks

The week-one and week-two `page` objects match, including all 39 atoms,
links, citations, changes, and rendered content. Their committed tick scopes
differ. Durable, processed, and verified ticks advance from `(1, 1, 1)` to
`(2, 2, 2)` without creating a page revision.

- Page revision:
  `30d4f2780746c3997195f6bb359b795d1714daf2249cef8f4933702e9bbce496`.
- Content SHA-256:
  `4ca00f3b9668e656b9c98cb22e0105293fa1aa1ebf402b1f7841f57424ed48e2`.
- Content effective tick: `1`.
- Week-one JSONL SHA-256:
  `0e022cd7ae8ffe61ddc4b202483528101138c6ee0c20ba6d014b7c8bcbde8f56`.
- Week-two and restarted JSONL SHA-256:
  `97aebf757430d0d66633da1b3059386a50d8834e7c806238df7ff4cbd92545c0`.

The CLI used the launcher's existing confined Archive login without changing
roles. The launcher passed credentials only in its child environment.

## Retained local evidence

Artifacts are under `/tmp/babylon-g4-archive-native-20260906/`:

- `test-launch.log` and `restart-launch.log`: commands, committed scopes,
  installed card state, and frame measurements.
- `05-week1-publishing.png`: despite its capture filename, this screenshot
  already shows the caught-up week-one card.
- `06-week2-held.png`: the same open card after the quiet week.
- `12-restart-archive.png`: the restored card after process restart.
- `week1.jsonl`, `week2.jsonl`, and `restart.jsonl`: confined CLI evidence.

## Limits

The requested resize to `1920×1080` remained `1366×768`, verified by
`xwininfo`, screenshots, and Bevy's physical-window measurements. Files named
`07-week2-1920.png` and `08-week2-after-resize.png` do not qualify the larger
resolution. The display mode was not changed in this session.

The quiet advance took about 9.99 seconds to durable acknowledgement.
The observer refreshed after 12.99 seconds, and the card caught up after 13.89 seconds.
Rendering remained approximately 60 FPS. These measurements include scheduling
and read work. They do not isolate SQL or simulation time.

Window-manager close establishes orderly shutdown, not keyboard-only Quit.
This operator-run check does not substitute for the Director's uninterrupted
bottleneck, dependency, worker, comparison, and resume comprehension session.
