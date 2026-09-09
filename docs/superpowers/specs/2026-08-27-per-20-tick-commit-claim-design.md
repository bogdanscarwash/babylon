<!-- Vale: this normative design preserves governed identifiers and wire terms. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
<!-- vale Vale.Terms = NO -->
<!-- vale ste.Gerunds = NO -->
<!-- vale ste.OneInstruction = NO -->
<!-- vale ste.PassiveVoice = NO -->
<!-- vale ste.Semicolon = NO -->
<!-- vale ste.SentenceLength = NO -->
<!-- vale strunk.ActiveVoice = NO -->
<!-- vale write-good.TooWordy = NO -->

# PER-20 Tick Commit Claim V1 Design

Status: approved first PER-20 implementation slice
Date: 2026-08-27
Linear: PER-20
Gate: Gate 3 persistence boundary, database-free contract slice

## Goal

Define the exact content-identity claim that a future `tick_commit` marker will
carry before Babylon adds PostgreSQL DDL or opens the Rust writer gate.

This slice connects the durable `CampaignId` and tick key to the existing
kernel-owned `TickContentHashV1`. It gives retry handling one closed answer:
an exact claim is idempotent, a different content identity at the same key is
a loud conflict, and a comparison across keys is invalid.

The claim is not `CommittedTickEnvelope`. It does not prove that state rows,
events, checkpoints, or outbox rows match. The later envelope composer must
validate those complete payloads before it can create a database transaction.

## Why this is first

The current H3 installer requires schema epoch 3. Adding migration 4 before
the installer sequence changes would break a fresh database. The current Rust
writer gate also correctly refuses authority while Python remains the sole
writer.

A database-free claim contract advances PER-20 without weakening either
boundary. It fixes the marker's campaign, tick, and content semantics before a
table or writer can accidentally make a different representation authoritative.

## Canonical layout

`TickCommitClaimV1` uses this exact byte sequence:

1. ASCII `babylon.tick-commit-claim` followed by NUL.
2. Big-endian `u32` layout version 1.
3. Tag `0x01`, then the 16 UUID bytes from `CampaignId` in network order.
4. Tag `0x02`, then the unsigned big-endian `u64` resolve tick.
5. Tag `0x03`, big-endian `u32` `TickContentHashV1` layout version 1, then
   the exact 32 digest bytes owned by `babylon-kernel`.

The total size is fixed at 93 bytes. The production type stores a fixed-size
array and performs no heap allocation. Production code composes this form
from typed values and exposes no decoder.

Campaign identity remains outside deterministic engine physics. Replay
session, replay seed, content digest, reference digest, accepted actions, and
the prior and result worlds already enter `TickContentHashV1`; the claim does
not copy or reinterpret them.

## Retry semantics

`classify_retry_against` treats `self` as the requested claim and its argument
as the existing claim.

- Different campaign or tick: `KeyMismatch`.
- Same key and same `TickContentHashV1`: `Idempotent`.
- Same key and different `TickContentHashV1`: `ContentIdentityMismatch`.

The method compares nominal typed fields. The language-neutral schema and
vectors prove that those fields reconstruct the exact fixed bytes. A future
database adapter must select the existing marker by `(campaign_id, tick)` and
apply this classification before it reports retry success.

## Ownership

- `babylon-persistence::identity::CampaignId` owns durable campaign identity
  and exposes its exact UUID bytes.
- `babylon-kernel::tick_content_hash::TickContentHashV1` remains the only tick
  content identity. Persistence adds no alias, re-export, or second hash.
- `babylon-persistence::tick_commit_claim` owns the claim layout and retry
  classification.
- `contracts/tick_commit_claim_v1.yaml` and its JSONL corpus own the
  language-neutral byte contract.

## Acceptance covered by this slice

- Campaign identity and replay identity remain separate typed concepts.
- `(campaign_id, tick)` is the durable marker key.
- The marker uses the exact PER-60 `TickContentHashV1` without re-hashing or
  decoding it.
- Exact content-identity retries succeed idempotently.
- Different content identity at the same key refuses loudly.
- The contract can represent a zero-row tick because it does not depend on a
  changed-state row.

## Explicit exclusions

This slice adds no migration, SQL, database connection, state row, event row,
checkpoint, hydration decoder, Archive receipt, outbox, commit acknowledgement,
failure injection, lost-tail replay, Python writer change, or Rust writer
authority. It does not claim full PER-20 acceptance.

The next slice must define the complete `CommittedTickEnvelope` payload and
its cumulative bounds before DDL can treat the claim as a `tick_commit` row.
That envelope must cover graph and state rows, events, subsystem rows,
conservation and boundary-flow rows, checkpoint data, and Archive dirty
receipts. Only then can same-payload idempotency mean whole-payload equality.

## Verification

The implementation uses RED-first Rust and Python tests. Both consumers
reconstruct canonical bytes from semantic UUID, tick, and digest inputs. They
cover minimum and maximum ticks, independent mutation of every field, exact
retry, content conflict, and key mismatch. The Python verifier independently
checks the YAML constants and rejects malformed or oversized vector input.
