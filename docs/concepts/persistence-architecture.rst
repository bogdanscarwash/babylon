Persistence Architecture
========================

Persistence begins only after the Rust tick judge succeeds. The engine crates
stay database-free, while ``babylon-persistence`` owns the one authoritative
game-managed PostgreSQL composition.

Why One Authoritative Backend?
------------------------------

The old Python design placed SQLite and PostgreSQL behind one runtime protocol.
That made authority depend on the selected adapter and allowed two game-state
storage paths to survive. Gate 3 replaces that model with an explicit split:

**Rust PostgreSQL authority**
   Owns campaign foundation, typed state, V2 events, choice receipts, commit
   markers, restart, and Archive dirty receipts. This boundary makes Rust the
   only production game-state writer.

**Python SQLite reference**
   ``RuntimeDatabase`` supports frozen behavioral tests and local reference
   work. It is not a deployment alternative and cannot acquire PostgreSQL
   authority.

**Dedicated periphery stores**
   Data, AI, documents, and vector search may use their own stores and
   credentials. Those credentials cannot mutate the governed game schemas.

This is a source-of-truth boundary, not a backend abstraction. There is no
adapter, fallback, compatibility view, or dual-write phase between Python and
Rust.

Judgment and Commit
-------------------

The sequence for one durable tick is:

#. ``ReplayTickSession`` judges the next tick on detached in-memory state.
#. The runtime prepares exact typed semantic rows and envelope bytes.
#. One Postgres transaction writes the action source and all six envelope
   families.
#. The transaction inserts ``babylon_state.tick_commit`` last with envelope
   layout 2.
#. Only an acknowledged ``COMMIT`` or exact ambiguity reconciliation publishes
   the receipt and caller sink.

``CommittedTickEnvelopeV2`` orders its families as graph, material state, event,
choice receipt, checkpoint, and Archive dirty receipt.

A rollback leaves no commit marker. An unresolved commit ambiguity leaves the
caller sink unchanged. Retry must reconstruct the same rows and envelope
identity. V2 event metadata records the emitting rule and an optional
engine-derived choice-receipt reference for a finite projection. Probability is
not part of an authored event payload, and event provenance cannot feed back
into mechanics.

Authority State Machine
-----------------------

The historical predecessor ledger admits ``prepared`` at schema epoch 8 and
``rust_active`` at epoch 9. The current
``babylon_meta.committed_tick_v2_authority_ledger`` then admits exactly two
rows: ``Prepared`` at activation epoch 10 and ``Active`` at epoch 11. The
prepared row binds the exact epoch 9 predecessor. The active row binds the
prepared-row digest and commits last during activation. Once active, only
``babylon-runtime`` can reacquire writer authority.

Epoch 9 retired the Python-managed relations only after ordered counts and
semantic hashes proved migration parity. Reachability analysis can instead
prove that the relations were empty.

Its destructive transaction uses
``READ COMMITTED``. Each present legacy relation and the closed opaque
predecessor set take ``ACCESS EXCLUSIVE`` locks before a fresh post-wait census.
Those locks stay held through disposition recording, deletion, the active
authority row, and commit. The census counts a writer that commits while
activation waits. The entire transaction then refuses without deleting the
writer's row.

Epoch 10 inventories every V1-incompatible campaign and tick relation and
refuses activation unless each is empty. Epoch 11 remains ``SERIALIZABLE``: its
top-level ``ACCESS EXCLUSIVE`` lock precedes every query and data modification,
so the runtime acquires its inventory snapshot after any lock wait. Epoch 11
removes the obsolete event relations and activates the V2-only reader and
writer. The terminal reader boundary contains no Python game-state edge, V1
envelope reader, or compatibility projection.

Restart Model
-------------

The runtime captures immutable foundation bytes for each campaign. A complete
full checkpoint contains the nine required state and identity sections. With no
committed marker, open starts from the foundation.

When committed ticks exist, restart loads the latest full checkpoint. It
reconstructs the six-family V2 envelope from typed rows and compares its digest
with the marker. Then it restores the nine checkpoint sections. It re-executes
each later marker-backed tick in a contiguous tail and compares the newly
prepared envelope with the stored envelope exactly.

Delta checkpoints reduce write volume but never act as restart roots. A gap,
duplicate, digest mismatch, or incomplete manifest refuses recovery rather than
guessing.

The Archive dirty receipt is one envelope family. The downstream semantic
Archive is not restart state: restart does not scan historical pages, grants,
consumptions, or citations and does not assert their historical integrity.

Reference Data and H3
---------------------

``babylon_ref`` owns fixed geography, H3 cohorts, and exact reference artifacts.
Artifact lookup uses the full digest-qualified database key. The runtime does
not fetch from ambient paths, the network, a latest-version alias, or a fallback
digest.

Rust installs and reads the canonical H3 products directly. Python continues to
build deterministic data artifacts, but it cannot write the authoritative
campaign estate.

Archive Boundary
----------------

Every committed tick writes exactly one Archive dirty receipt carrying the
tick-content identity. The receipt is durable evidence that semantic material
must refresh. Choice receipts separately preserve finite-realization evidence,
and V2 event metadata records its emitting rule and optional adjacent
choice-receipt provenance. ``sim:archive`` installs the additive client-owned
Archive schema exactly once. Later calls check its marker and relations. The
command reports receipts, knowledge grants, consumptions, and materialized
pages.

The first semantic worker slice binds an exact dirty batch, worker contract,
and ordered knowledge-grant snapshot to each marker-backed receipt. The
snapshot includes grant provenance, so an exact retry refuses knowledge drift.
It applies subject and field knowledge grants in SQL. It renders pages with the
pinned strict MiniJinja template, persists subject and visible-signal
citations, and searches only granted visible material. Later Gate 3 work adds
broader dirty-subject producers, dossier coverage, and the playable retrieval
loop. This slice alone does not pass a game milestone.

PER-23 consumes one stable known-only retrieval boundary and must not re-read
Archive, grant, or ledger tables to build its dossier view.
``SemanticArchiveStoreV1::search_known`` is the only retrieval path, with no
raw-ledger fallback. One hit is self-contained. It carries the rendered page
and title, the honest ``verified_tick``, and the subject kind and identity.
It also carries the visible-signal content rendered into the page and the
page's provenance citations with exact source and locator. A read checks
stored page bytes against their content digest, and one result set stays
within the search bound.

Operational Consequences
------------------------

- ``db:bootstrap`` performs the idempotent Rust activation.
- ``sim:e2e-michigan`` runs a fresh Rust-owned durable campaign.
- ``sim:probe`` reports the selected worktree campaign tail and labels
  database-wide totals separately.
- ``sim:archive`` installs the additive semantic Archive schema or checks its
  marker and relations. It reports the Archive estate.
- ``qa:michigan-rollover-smoke`` proves a 60-tick restart boundary.
- The live adopter suite uses disposable pinned Postgres 17 and PostGIS 3.5.
- The suite verifies rollback, lock refusal, ambiguous commit, installed
  mutations, and cleanup.

See Also
--------

- :doc:`/reference/persistence` — Commands, types, and contract references
- :doc:`/concepts/architecture` — Whole-system live boundary
- :doc:`/reference/determinism-contract` — Hash and replay identity contracts
