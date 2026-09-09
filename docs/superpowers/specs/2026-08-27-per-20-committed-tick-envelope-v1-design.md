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
<!-- vale ste.Modals = NO -->
<!-- vale ste.Dictionary = NO -->
<!-- vale strunk.ActiveVoice = NO -->
<!-- vale write-good.TooWordy = NO -->

# PER-20 `CommittedTickEnvelopeV1` Design

**Issue:** PER-20
**Status:** Implemented database-free contract
**Decision:** ADR243

## Objective

Define the smallest complete container that can represent every logical output
of one future atomic tick transaction. Make same-payload retry mean exact
whole-payload equality before Babylon adds PostgreSQL DDL or opens the Rust
writer gate.

This slice follows `TickCommitClaimV1`. The claim already binds `CampaignId`,
resolve tick, and the kernel-owned `TickContentHashV1`. It does not bind the
rows that a future transaction will write. The envelope closes that gap without
claiming database durability.

## Authority boundary

`babylon-persistence` owns the envelope framing, cumulative bounds, and retry
classification. It consumes `TickCommitClaimV1` directly. It does not alias,
re-export, decode, or recompute `TickContentHashV1`.

The envelope row is intentionally semantic-free. It owns exact canonical key
and payload bytes after a typed family producer creates them. The family
producer still owns material meaning, field types, units, and canonical row
encoding. The envelope cannot turn arbitrary bytes into an authoritative state
row, and this slice does not invent those codecs before the schema mapping is
designed.

## Mandatory payload

Every V1 envelope contains these sections in one closed order:

1. Graph rows.
2. Material state rows.
3. Event rows.
4. Subsystem rows.
5. Conservation rows.
6. Boundary-flow rows.
7. Checkpoint rows.
8. Archive dirty-receipt rows.

Each section is present when empty. This rule lets a zero-changed-row tick keep
an exact commit claim and prevents a missing family from becoming ambiguous
with an empty family.

Rows use a nonempty canonical primary key and an exact canonical payload.
Keys must ascend strictly within one family. The composer refuses an empty key,
a duplicate key, or descending input. It does not sort silently because caller
order is part of the boundary evidence and a hidden sort could mask a producer
fault.

## Canonical V1 framing

The complete byte sequence uses big-endian unsigned lengths:

- `babylon.committed-tick-envelope.v1` plus a mandatory NUL byte.
- Envelope layout `u32` value `1`.
- Claim tag `0x01`, claim length `u32` value `93`, and the exact existing claim.
- Eight family sections with tags `0x10` through `0x17`.
- Each family has a `u32` row count, a `u32` body length, and exact ordered rows.
- Each row has a `u32` key length, key bytes, `u32` payload length, and payload
  bytes.

The framing has no production decoder in this slice. The future writer receives
the checked immutable object. The independent verifier reconstructs bytes only
to prove that another implementation can reproduce the contract.

## Cumulative bounds

The aggregate row ceiling is 1,048,576. It reuses the existing governed tick
aggregate ceiling. One family body is at most 67,108,864 bytes, which reuses the
existing governed identity-section ceiling. A body includes both four-byte row
lengths plus the exact key and payload bytes. Since keys cannot be empty, a
preflight body must contain at least nine bytes for every declared row.

The fixed envelope framing is 209 bytes. Eight maximum family bodies therefore
produce an exact maximum complete length of 536,871,121 bytes. The public
preflight accepts the reachable maximum shape and refuses maximum plus one
without allocating hundreds of megabytes. The actual composer repeats the same
checks before it allocates the exact canonical capacity.

## Retry classification

Retry classification has four closed outcomes:

- A different campaign or tick returns the existing claim's key mismatch.
- The same key with a different `TickContentHashV1` returns the existing claim's
  content-identity mismatch.
- The same claim and the same exact complete bytes is idempotent.
- The same claim and any different complete byte returns
  `WholePayloadMismatch`.

`CommittedTickEnvelopeDigestV1` is a SHA-256 diagnostic for logs and evidence.
The retry decision compares exact canonical bytes. A digest collision cannot
turn different payloads into an idempotent retry. The envelope diagnostic also
cannot substitute for `TickContentHashV1`, which remains the constitutional
replay-tick identity.

## Explicit exclusions

This slice adds no semantic state, event, subsystem, conservation,
boundary-flow, checkpoint, or Archive row codec. It adds no PostgreSQL schema,
migration, connection, transaction, marker, commit acknowledgment, hydration,
crash injection, or lost-tail replay. It does not activate the Rust writer,
change the Python writer, process Archive receipts, execute actions, modify BSL,
or change a material mechanic.

The next PER-20 work must define typed row codecs and their schema mappings
before it can construct a real transaction. Marker-last execution, crash
injection, hydration, bounded PostgreSQL-unavailable behavior, and one-way
writer cutover remain later acceptance work.

## Verification

The shared YAML schema and JSONL corpus pin the exact family order, tags,
framing, bounds, exclusions, zero-row encoding, and all four retry outcomes.
Rust reconstructs the shared vectors through the production composer. The
independent Python verifier reconstructs the same bytes from semantic values.

Both paths cover every family mutation, empty, duplicate, and descending keys,
the exact cumulative maxima, and maximum-plus-one refusals. The scoped Rust
tests, clippy with warnings denied, Python tests, formatter, repository checks,
and exact-head CI remain required before merge.
