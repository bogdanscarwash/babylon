<!-- Vale: this normative design preserves governed identifiers and wire terms. -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.UnapprovedWords = NO -->
<!-- vale ste.NounClusters = NO -->
<!-- The literal APIs, layouts, and failure predicates require the same local
     heuristic exemptions as the repository's other executable designs. -->
<!-- vale Vale.Terms = NO -->
<!-- vale ste.Ambiguity = NO -->
<!-- vale ste.Dictionary = NO -->
<!-- vale ste.Gerunds = NO -->
<!-- vale ste.LatinAbbrev = NO -->
<!-- vale ste.Modals = NO -->
<!-- vale ste.OneInstruction = NO -->
<!-- vale ste.PassiveVoice = NO -->
<!-- vale ste.ProcedureLength = NO -->
<!-- vale ste.Semicolon = NO -->
<!-- vale ste.SentenceLength = NO -->
<!-- vale ste.ThisPronoun = NO -->
<!-- vale strunk.ActiveVoice = NO -->
<!-- vale strunk.CommonlyMisused = NO -->
<!-- vale write-good.ThereIs = NO -->
<!-- vale write-good.TooWordy = NO -->
<!-- vale write-good.Weasel = NO -->

# PER-60 Replay, Action, and Tick Content Identity Design

Status: Director-authorized design for implementation
Date: 2026-08-26
Linear: PER-60
Gate: Gate 3 database-free identity boundary

## Goal

PER-60 gives one detached Rust tick a complete, versioned, language-neutral
identity. The result is the input boundary that PER-20 will later place inside
CommittedTickEnvelope and persist atomically.

This work serves the playable slice. It makes the future player decision loop
replayable and auditable. It does not activate player actions, persistence,
Archive delivery, or dynamic graph topology.

The implementation succeeds when an identified tick binds all of these inputs
and outputs:

- the replay session and replay seed;
- the RNG layout;
- the content and reference identities;
- the exact prepared mechanics environment;
- the stable world before and after adjudication;
- the ordered accepted-action batch, which is empty at runtime in Gate 3;
- the exact ordered tick payload, including events and audit receipts; and
- every nested layout version needed to interpret those values.

The same semantic tick must produce the same canonical bytes and SHA-256 digest
across processes, graph backends, and runtime-handle allocation orders.

## Current facts

The following statements describe live behavior at the design base,
origin/dev commit 600e6b03cd3eff271cd4ccaecf5a914327d47581.

- Python P27 has executable canonical bytes and pinned vectors.
- Rust KernelRng uses ChaCha8 with one stream per stable carrier. Its current
  seed derivation uses SessionId, tick, domain, stable key, and a fixed salt.
  It has no replay seed.
- TickSession advances a detached graph atomically and publishes events,
  identity-free AuditReceipt rows, completed time, GraphStateHash, and
  NominalWorldHash.
- GraphStateHash encodes runtime NodeId and HyperedgeId values.
  NominalWorldHash nests GraphStateHash and allocator cursors. Neither hash can
  enter the authoritative TickContentHash.
- The scenario loader already retains insertion-independent node names. It
  also builds insertion-independent hyperedge names during hydration, but
  discards that inverse map before returning.
- ResolvedPracticeBatchV2 already validates complete PracticeProposalKeyV2
  order. PracticeIntentV2 already binds actor, target, practice verb, proposal
  nonce, quoted contracts, parameters, and evidence.
- Executable intents and player actions do not exist. Gate 5 owns their
  activation.

## Director rulings

### P27 disposition

P27 remains executable and byte-pinned as frozen reference evidence. Its existing
vectors must not change.

P27 bytes and digests do not enter authoritative TickContentHashV1. The new
Rust binary contract is the sole authoritative tick-content identity.

This ruling supersedes only prior wording that required the modern Rust
TickContentHash to equal P27 bytes. It does not retire P27 or weaken its tests.

### Delivery boundary

PER-60 implements the exact empty accepted-action batch in the live replay
tick. It may implement and vector-test the non-empty action codec, because that
codec defines ActionId and canonical order. It must not admit, adjudicate, or
execute a non-empty action batch.

PER-20 retains PostgreSQL schemas, hydration, CommittedTickEnvelope, outbox
publication, and writer cutover. Gate 5 retains player input and action
effects.

## Canonical primitive rules

All new PER-60 integers use fixed-width big-endian bytes. SHA-256 digests use
32 raw bytes. Strings use exact UTF-8 bytes after validation and never use
JSON, locale-sensitive formatting, or unordered map iteration.

ReplaySessionIdV1 is the authoritative session form. It contains from 1
through 256 ASCII graphic bytes in the inclusive range `0x21` through `0x7e`
and encodes as:

    u16 byte length
    exact ASCII bytes

ASCII makes normalization tables and Unicode-version drift irrelevant. The
implementation rejects spaces, control bytes, DEL, and non-ASCII input; it
never transforms case or bytes. The existing SessionId remains the current
TickSession logging and RNG V1 identity. The replay path validates a distinct
ReplaySessionIdV1 before adjudication.

ReplaySessionIdV1 is a logical replay namespace, not a rendering of
CampaignIdV2, babylon-persistence CampaignId, a database row id, or another
durability identity. No `From` or `TryFrom` implementation connects those
types, and the authoritative composition root cannot accept one in place of a
replay session. A string that happens to resemble a UUID does not change its
type or provenance. PER-20 must persist campaign identity and replay session
as separate typed fields and must supply them separately to persistence and
the replay engine.

Every collection has a declared fixed ceiling. Every integer conversion is
checked. NaN and infinity refuse. Canonical floating-point encoders collapse
negative zero to positive zero and otherwise write exact IEEE-754 bits.

The layouts below use `str32(value)` for `u32` byte length followed by exact
UTF-8 bytes. A semantic category can impose a lower limit than `u32`:

- a BSL symbol or authored local name is 1 through 64 strict ASCII bytes;
- a BSL qname is 1 through 128 strict ASCII bytes and at most four segments;
- a structural node, edge, or hyperedge type member and a canonical full event
  type are each 1 through 128 strict ASCII bytes at the replay boundary;
- an intrinsic identity name is 1 through 96 strict ASCII bytes under its
  existing delimiter rules; and
- an enum type and member are each 1 through 64 strict ASCII bytes under their
  existing case-specific grammars; and
- a governance string without a narrower live grammar limit is at most
  4,194,304 UTF-8 bytes at the replay boundary.

Encoders and verifiers apply the lower semantic ceiling first. They reject an
object over its total-byte ceiling before any new PER-60 codec-buffer
allocation, validate a count before reserving codec rows, use checked
aggregate-row arithmetic, and reject trailing bytes. Existing CanonicalState
listing methods materialize their seven Vec values before the stable encoder
sees them; changing that substrate API is outside this slice. Process-level
out-of-memory abort is not a recoverable codec error. These are admission
limits for replay identity, not claims that an otherwise unbounded source
token already had the same limit.

## Replay seed and RNG layouts

ReplaySeed is a signed i64 over the complete i64 range. Zero and negative
values are valid when supplied explicitly.

Its canonical form is i64.to_be_bytes. Its future PostgreSQL mapping is
BIGINT NOT NULL with no default. A value outside the i64 range refuses before
adjudication.

RngLayoutVersion has two governed values:

- V1 freezes the current seed_for and KernelRng behavior byte for byte. It is
  the current TickSession contract, ignores ReplaySeed, and is not eligible for
  a durable Gate 3 campaign.
- V2 is the seed-aware replay layout. Every new replay session uses V2.

RNG V2 derives one 32-byte ChaCha8 seed per carrier from this exact preimage:

    ASCII "babylon.rng-stream" followed by NUL
    u32 layout version 2
    tag 0x01, ReplaySeed i64 big-endian
    tag 0x02, ReplaySessionIdV1 canonical field
    tag 0x03, resolve tick u64 big-endian
    tag 0x04, u32 byte length and exact UTF-8 domain
    tag 0x05, u32 byte length and exact UTF-8 stable carrier key

SHA-256 of that preimage is the exact 32-byte ChaCha8 key. RNG V2 freezes the
rand_chacha 0.10 output sequence as a language-neutral algorithm, independent
of a future Rust dependency implementation:

1. Form sixteen u32 state words. Words 0 through 3 are `0x61707865`,
   `0x3320646e`, `0x79622d32`, and `0x6b206574`. Words 4 through 11 decode
   consecutive four-byte key chunks as little-endian u32. Words 12 and 13 are
   the low and high u32 halves of a zero-based 64-bit block counter. Words 14
   and 15 are a zero 64-bit stream id.
2. For each 64-byte block, copy the initial state and run four double rounds.
   One quarter round `(a,b,c,d)` uses wrapping u32 addition and performs:
   `a+=b; d^=a; d=rotl(d,16); c+=d; b^=c; b=rotl(b,12); a+=b; d^=a;
   d=rotl(d,8); c+=d; b^=c; b=rotl(b,7)`.
3. Each double round applies quarter rounds first to columns
   `(0,4,8,12)`, `(1,5,9,13)`, `(2,6,10,14)`, `(3,7,11,15)`, then to
   diagonals `(0,5,10,15)`, `(1,6,11,12)`, `(2,7,8,13)`, and
   `(3,4,9,14)`.
4. Add each original state word to its working word with wrapping u32
   addition. The sixteen result words appear in index order and each word's
   byte form is little-endian. Increment the 64-bit block counter by one with
   wrapping arithmetic. The stream id remains zero.
5. next_u64 consumes two consecutive result words: the first is bits 0 through
   31 and the second is bits 32 through 63. next_f64 consumes one next_u64,
   shifts it right by 11, converts that 53-bit integer exactly to f64, and
   multiplies by exact `2^-53`.

The API offers no stream-id setter, seek operation, or tick-global stream.
An implementation can call rand_chacha 0.10, but the algorithm and vectors
above own reproducibility.

An unknown layout refuses. A future RNG change creates V3 beside V2 and gets
new vectors and a new ceremony. It never edits V2 in place.

V1 keeps its existing unversioned preimage exactly: session UTF-8 bytes,
little-endian tick, little-endian `0x0BA1_AC1A`, little-endian `u64` domain
length and bytes, then little-endian `u64` key length and bytes. V1 gains no
new validation, normalization, length cap, seed, or prefix.

V2 introduces two validated boundary values:

- RngDomainV2 is the firing rule qname: 1 through 128 strict ASCII bytes.
- StableCarrierKeyV2 is a graph-owned private strict ASCII value of 1 through
  105,962 bytes. The authoritative BSL path cannot construct it from an
  arbitrary caller string.

The stable carrier uses graph-owned StableElementCarrierSegmentV1, an exact
ASCII rendering of the same validated semantic key as binary
StableElementKeyV1. It is a second codec owned beside the binary codec, not the
binary key bytes and not a BSL reimplementation. Each textual segment is
decimal byte length, `:`, then exact bytes, and `|` separates segments:

    node: framed("node", scenario qname, authored local name)
    hyperedge: framed("hyperedge", scenario qname, authored local name)
    edge: framed("edge", scenario qname, edge type,
                 source local name, target local name)

The graph-owned builder takes one resolved subject segment, at most 256
resolved active-element segments in outermost-to-innermost order, and the
canonical decimal i64 draw slot. The slot has no leading plus or zeroes; zero
is `0`. The builder checks its segment count and 105,962-byte ceiling before
allocation. V2 has no raw-id or debug fallback.

The final bytes are exactly:

    StableCarrierKeyV2 = framed(
      subject_segment.ascii(),
      active_segments[0].ascii(), ... in outermost-to-innermost order ...,
      canonical_slot_ascii)

The outer `framed` call applies the same decimal-byte-length, colon, and pipe
framing again; each already-framed element rendering is one opaque ASCII
segment at this level. With zero active elements, the exact form is
`framed(subject_segment.ascii(), canonical_slot_ascii)`.

The crate dependency direction is explicit. BSL resolves its subject and
element stack through graph, asks graph to build StableCarrierKeyV2, and then
passes only `validated_bytes()` to the low-level kernel V2 derivation. Kernel
cannot name a graph type because graph already depends on kernel, so its raw
byte entry point does not by itself prove provenance. ReplayTickSession's
typed BSL path is the authoritative construction boundary; direct low-level
kernel use is standalone contract-test evidence, never tick execution.

One typed RngSeedContext dispatches the real intrinsic path:

- V1 contains SessionId and calls the unchanged V1 API;
- V2 contains ReplaySessionIdV1 and ReplaySeed and calls only seed_for_v2; and
- DrawContext carries the typed context, so `rng-draw` cannot ignore the V2
  seed while a separate helper happens to hash it correctly.

ReplayTickSession accepts only the V2 context. The current TickSession constructs
V1. The numeric layout is parsed once in babylon-kernel; BSL and tick do not
duplicate a `version == 2` branch.

## Ordered Practice action identity

PER-60 adds OrderedPracticeActionBatchV1. It is deliberately specific to the
accepted Practice V2 contract. It is not a generic action framework.

### Stable actor identity

ActorOrganizationIdV2 is an opaque eight-byte domain identity with a private
field. Its only primitive construction and projection are from and to exact
`[u8; 8]` bytes. It has no arithmetic, numeric conversion, NodeId conversion,
or graph dependency.

Every Practice V2 actor lane uses this type: authority rows and lookup,
PracticeIntentV2, PracticeProposalKeyV2, resource ownership, and strike
organization relations. Its existing wire form remains the same eight bytes
previously written by `u64::to_be_bytes`, and unsigned bytewise ordering stays
the same. Existing V2 vectors therefore do not move. V1 actor fields remain
unchanged.

This is a Rust API and type-safety refinement over the frozen eight-byte
`u64_be` wire slot. The accepted Practice V2 YAML files, their pinned source
SHA constants, canonical encoders and decoders, JSONL vectors, and vector
manifests remain byte-identical. They keep the existing `actor_org_id_u64` and
`type: u64` wire descriptions. Numeric JSON fixture adapters may convert a
parsed u64 through `to_be_bytes` solely to construct the Rust newtype. A
schema-semantic or wire change requires a new Practice contract version and
ceremony; it cannot land under ActorOrganizationIdV2.

Gate 5 must provide an explicit stable actor-to-graph resolver before an actor
can affect graph mechanics. A cast from ActorOrganizationIdV2 to NodeId is not
such a resolver.

### Proposal and canonical order

PracticeProposalKeyV2 remains proposal identity:

    resolve tick
    input authority id
    actor organization id
    practice id
    tagged stable target identity
    proposal nonce

For a non-empty structural projection, ResolvedPracticeBatchV2 validation runs
first against the supplied trusted authority ledger. Its items already form
one strictly ascending and unique proposal-key sequence. The Gate 3 empty form
needs no campaign or authority-ledger input.

The projector enumerates that sequence and assigns a contiguous zero-based
u16 `canonical_input_ordinal`. This is the input ordinal required by PER-60.
The existing 4,096 item ceiling limits valid ordinals to 0 through 4,095.

The ordinal is an order witness and event or receipt correlation position. It
is never initiative, scarcity priority, execution rank, or proposal identity.
Shared material scarcity must remain order-neutral under ADR236.

### ActionId

Each accepted item reuses the exact PracticeIntentV2 canonical bytes. The
nested intent supplies:

- actor through actor_org_id;
- target through TaggedPracticeTargetV2;
- verb through PracticeIdV2; and
- payload identity through practice_intent_v2_digest.

actor_org_id is ActorOrganizationIdV2, not a numeric graph handle.

ActionId is SHA-256 of:

    ASCII "babylon.practice-action-id.v1" followed by NUL
    u16 action-id schema version 1
    ReplaySessionIdV1 canonical field
    u16 nested PracticeIntent schema version 2
    32-byte practice_intent_v2_digest

ActionId excludes canonical_input_ordinal, campaign UUID, database identity,
wall time, and ActionId itself. It therefore remains stable when an unrelated
lower-sorting action changes later ordinals.

### Batch bytes

OrderedPracticeActionBatchV1 encodes:

    ASCII "babylon.ordered-practice-action-batch.v1" followed by NUL
    u16 schema version 1
    ReplaySessionIdV1 canonical field
    u64 resolve tick
    u16 item count
    for each item in canonical order:
      u16 canonical_input_ordinal
      32-byte ActionId
      u16 intent byte length
      exact PracticeIntentV2 canonical bytes

Validation requires ordinal equality with array position, one resolve tick,
strict ascending unique proposal keys, recomputed ActionId values, and bounded
intent lengths. The batch digest is SHA-256 of the validated canonical batch
bytes and never appears inside its own preimage.

OrderedPracticeActionV1 and OrderedPracticeActionBatchV1 have private fields.
The empty constructor is the only live Gate 3 constructor. The non-empty
projector takes ReplaySessionIdV1, the complete ResolvedPracticeBatchV2, and
the trusted PracticeInputAuthorityLedgerV2. It first runs the existing full
resolved-batch validation, then clones exact intents, derives ordinals and
ActionId values, and constructs the private batch. There is no constructor
from loose intents, caller ordinals, caller ActionId values, or raw bytes.

Resolved-batch validation proves structural and active-authority consistency
relative to the supplied ledger. It does not prove submission history,
resource allocation, or an accepted-input transaction; ADR235 leaves those
owners outside that contract. The non-empty PER-60 vector is therefore a
codec projection fixture, not evidence of live admission. Gate 5 and PER-20
must supply accepted-input and commit provenance before a non-empty projection
can become authoritative. A future rehydrator must replay the projector from
the source batch and an independently trusted committed ledger, compare exact
bytes, and return the recomputed value. Raw ordered bytes never confer
acceptance.

Exact action bounds are:

- ActionId preimage is `68 + session byte length`, or 69 through 324 bytes;
- an empty batch is `55 + session byte length`, or 56 through 311 bytes;
- each valid current Practice V2 item is at most 2,271 bytes; and
- the complete 4,096-item batch is at most 9,302,326 bytes.

Practice V2 keeps its frozen 16,384-byte decoder ceiling. Its current valid
empty-parameter grammar can produce at most 2,235 intent bytes, so PER-60 does
not restate the wider decoder ceiling as an ordered-action operation bound.

ReplayTickSession accepts the typed batch but requires the zero-item form. A
non-empty form refuses before the detached tick starts. Gate 5 must change this
explicit guard before any action can enter mechanics.

### Refused and material outcomes

A malformed or pre-admission-refused attempt receives no ActionId, ordinal,
action event, receipt, or world-identity contribution. When decodable, its
proposal key may identify the refusal outside authoritative tick content.

An accepted action retains its ActionId for applied, partial, no-participation,
materially failed, countered, or accepted no-op outcomes. Those outcomes will
be authoritative payload rows after Gate 5 defines their schema.

Canonical order may order serialized actions, events, and receipts. It must
not decide a material winner. A noncommutative contention must use an explicit
governed causal law or refuse.

## Stable graph and world identity

PER-60 adds one graph-owned stable identity path beside GraphStateHash. BSL
supplies authored names; graph code owns validation, key bytes, state bytes,
and hashes. Tick and BSL call that one resolver rather than re-encoding keys.

### Stable element keys and resolver

StableElementKeyV1 has these governed kind bytes and standalone encoding:

    ASCII "babylon.stable-element" followed by NUL
    u32 layout version 1
    u8 kind: 0x01 node, 0x02 directed dyadic edge, 0x03 hyperedge
    for node:
      str32 scenario scope
      str32 authored node local name
    for edge:
      str32 scenario scope
      str32 edge type
      str32 source node local name
      str32 target node local name
    for hyperedge:
      str32 scenario scope
      str32 authored hyperedge local name

Node and hyperedge type are world facts, not identity components. An edge has
no authored local name, so its type and ordered endpoints identify it. The
largest standalone key is 428 bytes under the 128-byte structural-type replay
ceiling. Parallel hyperedges remain distinct through authored local names,
even when type and members match.

StableElementCarrierSegmentV1 is the graph-owned ASCII renderer used only
inside StableCarrierKeyV2. It renders the same validated semantic key as:

    node: framed("node", scenario scope, authored node local name)
    edge: framed("edge", scenario scope, edge type,
                 source node local name, target node local name)
    hyperedge: framed("hyperedge", scenario scope,
                      authored hyperedge local name)

It is not the standalone binary StableElementKeyV1 bytes. Graph code owns and
tests both codecs so BSL cannot make their field identity drift.

StableElementResolverManifestV1 encodes the stable resolver inputs without
runtime handles:

    ASCII "babylon.stable-element-resolver" followed by NUL
    u32 layout version 1
    tag 0x01, str32 scenario scope
    tag 0x02, u32 node count
      rows sorted by local name: str32 local name, str32 node type
    tag 0x03, u32 hyperedge count
      rows sorted by local name: str32 local name, str32 hyperedge type

All three tags are mandatory. Node local names are unique within the node
section, and hyperedge local names are unique within the hyperedge section.
The combined node and hyperedge count is at most 65,536, one hyperedge has at
most 65,534 members, all topology rows and member references total at most
1,048,576 fact units, and the manifest is at most 8,388,608 bytes. Its digest
is SHA-256 of the exact bytes.

The immutable StableElementResolverV1 also seals the hydrated topology. It
requires a bijection between live node handles and authored node names and a
second bijection between live hyperedge handles and authored hyperedge names.
Every indexed edge must have two named live endpoints. Missing, extra,
duplicate, dangling, or non-ASCII identities refuse. Any added or removed
node, edge, hyperedge, or membership after sealing refuses.

This fixed-topology contract is viable only because all six graph-shape verbs
remain load-refused. No lookup may fall back to Debug, a decimal runtime id,
insertion ordinal, type-plus-members synthesis, or allocator state.

### Stable graph bytes

StableGraphStateV1 consumes the same seven CanonicalState fact listings as the
current graph hash, resolves every runtime handle, and encodes:

    ASCII "babylon.stable-graph" followed by NUL
    u32 layout version 1
    tag 0x01, str32 scenario scope
    tag 0x02, u32 node count
      str32 node local name, str32 node type
    tag 0x03, u32 node f64 attribute count
      str32 node local name, str32 qname, u64 canonical f64 bits
    tag 0x04, u32 dyadic edge count
      str32 edge type, str32 source local name, str32 target local name,
      u64 canonical strength bits
    tag 0x05, u32 hyperedge count
      str32 hyperedge local name, str32 hyperedge type, u32 member count,
      then str32 member local names
    tag 0x06, u32 edge f64 attribute count
      str32 edge type, str32 source local name, str32 target local name,
      str32 qname, u64 canonical f64 bits
    tag 0x07, u32 node Currency attribute count
      str32 node local name, str32 qname, i128 micro-units
    tag 0x08, u32 hyperedge f64 attribute count
      str32 hyperedge local name, str32 qname, u64 canonical f64 bits

All eight tags are mandatory, exactly once, in order. Every listing section
writes its count even when empty; this layout inherits no prior empty-section
elision. StableGraphStateHashV1 is SHA-256 of these exact bytes.

Canonical unsigned-ASCII order is local name for nodes and hyperedges;
`(node, qname)` for both node-attribute lanes; `(type, source, target)` for
edges; `(type, source, target, qname)` for edge attributes; and `(hyperedge,
qname)` for hyperedge attributes. Hyperedge members sort by node local name.
Values never break a key tie. Equal keys refuse.

The encoder also refuses an unknown attribute owner, an absent exact edge, an
empty hyperedge, an unknown or duplicate member, a duplicated fact row, one
`(node, qname)` present in both numeric lanes, or an edge-attribute
`/strength` row. Edge strength has its only home in tag 0x04. BSL retains
semantic field-owner and type validation; graph identity does not duplicate
the BSL typechecker.

V1 ceilings are:

    stable nodes, edges, or hyperedges each: 65,536
    rows in any attribute section: 524,288
    members in one hyperedge: 65,534
    all seven rows plus nested member references: 1,048,576 fact units
    complete StableGraphStateV1 bytes: 67,108,864

Counts and byte sizes use checked arithmetic before reserve or append.
Finite f64 values normalize either zero to positive zero and encode exact
big-endian bits. Currency writes its signed i128 micro-unit value.

### World-register seam

The only live mutable authoritative non-graph register is completed tick. It
gets one explicit manifest rather than a list of speculative future fields.

WorldRegisterManifestV1 encodes:

    ASCII "babylon.world-register-manifest" followed by NUL
    u32 manifest version 1
    u32 entry count 1
    str32 ASCII "world/completed-tick"
    u32 register layout version 1

WorldRegisterSetV1 encodes:

    ASCII "babylon.world-register-set" followed by NUL
    u32 set layout version 1
    tag 0x01:
      u32 manifest version 1
      32-byte WorldRegisterManifestV1 digest
    tag 0x02:
      u32 entry count 1
      str32 ASCII "world/completed-tick"
      u32 register layout version 1
      u32 payload length 8
      non-negative completed tick as i64 big-endian

WorldRegisterSetDigestV1 is SHA-256 of the exact set bytes.

StableWorldV1 encodes:

    ASCII "babylon.stable-world" followed by NUL
    u32 layout version 1
    tag 0x01:
      u32 StableGraphState layout version 1
      32-byte StableGraphStateHashV1
    tag 0x02:
      u32 WorldRegisterSet layout version 1
      32-byte WorldRegisterSetV1 digest

The prepared environment binds the same register-manifest version and digest.
Prior and result register sets must match it exactly. Missing, extra,
duplicate, out-of-order, unknown-version, or trailing data refuses.

StableWorldDigestV1 is SHA-256 of the exact StableWorldV1 bytes. For an
identified resolve tick from 1 through i64 maximum, the prior completed-tick
register equals resolve tick minus one and the result register equals resolve
tick. Both conversions and the subtraction are checked.

Allocator cursors stay out. Phase schedule belongs to the prepared environment;
events and receipts belong to the tick payload; seed, session, content,
reference, and action identity belong to the outer tick hash. A real future
register gets a named payload layout and a new manifest version when it becomes
live. PER-60 does not predeclare inventory, reservation, knowledge, mint, or
other future placeholders.

GraphStateHash and NominalWorldHash remain unchanged as current contracts for
administrative display. TickContentHashV1 never nests either one.

## Prepared environment identity

PreparedEnvironmentV1 commits the mechanics that the engine actually loaded,
without encoding Rust HashMap order or runtime graph handles.

### Shared BSL discriminants

These tag assignments are governed wire data and never derive from Rust enum
order:

    ValueV1:
      0x01 Int, then i64
      0x02 Currency, then signed i128 micro-units
      0x03 Real, then canonical f64
      0x04 Ratio, then canonical f64 value,
           u8 optional exclusive-floor presence and optional f64,
           u8 optional inclusive-cap presence and optional f64
      0x05 Bool, then u8 0 or 1
      0x06 Enum, then str32 enum type and str32 member
      0x07 NodeRef, then exact node-kind StableElementKeyV1 bytes
      0x08 HyperedgeRef, then exact hyperedge-kind StableElementKeyV1 bytes
      0x09 EdgeRef, then exact edge-kind StableElementKeyV1 bytes

    BslTypeV1:
      0x01 Probability, 0x02 Intensity, 0x03 Coefficient,
      0x04 Currency, 0x05 Real, 0x06 Int, 0x07 Bool,
      0x08 Enum, then str32 resolved enum type name,
      0x09 NodeSet, then str32 NodeType member,
      0x0a EdgeSet, then str32 EdgeType member

    FieldKindV1:
      0x01 Intensive, 0x02 Extensive, 0x03 NotApplicable

    RuleRoleV1:
      0x01 Mechanic, 0x02 Recognizer, 0x03 ExternalEvent, 0x04 Intent

    EvidenceClassV1:
      0x01 Observed, 0x02 Derived, 0x03 Calibrated, 0x04 Designed

    EffectSignatureV1:
      0x01 NodeField, then str32 qname
      0x02 EdgeField, then str32 qname
      0x03 HyperedgeField, then str32 qname
      0x04 Event, then str32 canonical full EventType/MEMBER
      0x05 Shape, then one u8 ShapeVerbV1 code

    ShapeVerbV1:
      0x01 AddNode, 0x02 RemoveNode, 0x03 AddEdge,
      0x04 RemoveEdge, 0x05 AddHyperedge, 0x06 RemoveHyperedge

    EnumKindV1:
      0x01 NodeType, 0x02 EdgeType, 0x03 HyperedgeType, 0x04 EventType

An option byte is exactly 0 for absent or 1 followed by its value. Any other
byte refuses. ConstValueV1 permits only ValueV1 tags 0x01 through 0x05, which
are exactly the live constant forms. An enum or reference constant is an
invariant failure. BslTypeV1 resolves EnumTypeId to its declared type name and
refuses an unknown id.

babylon-bsl owns these codecs, registry snapshot helpers, and their semantic
validation. babylon-tick composes their returned checked sections; it does not
maintain a second tag table.

### Governed phase schedule

PreparedEnvironmentV1 reuses the existing PhaseScheduleV1 bytes:

    ASCII "babylon.phase-schedule" followed by NUL
    u32 layout version 1
    u32 canonical slot count
    for each slot in governed order:
      str32 name
      u8 partition: 0 material base, 1 action, 2 consequence
      u8 ordinal
      u16 default rank
    u32 accepted-name mapping count
    for each mapping sorted by accepted name:
      str32 accepted name
      str32 canonical slot
      u16 resolved default rank

Its current 34 slots and four governed accepted-name mappings remain the live
schedule. Each accepted name is used by current content; this is not a
second scheduling path.
PhaseScheduleDigestV1 is SHA-256 of the exact bytes.

### Prepared environment bytes

PreparedEnvironmentV1 encodes these fixed sections exactly once and in order:

    ASCII "babylon.prepared-environment" followed by NUL
    u32 layout version 1
    tag 0x01:
      32-byte verified rules_hash
    tag 0x02:
      u32 PhaseSchedule layout version 1
      32-byte PhaseScheduleDigestV1
    tag 0x03:
      u32 rule count
      str32 rule qnames in resolved execution order
    tag 0x04:
      u32 field count
      rows sorted by qname:
        str32 qname, BslTypeV1, FieldKindV1
      u32 exemption count
      rows sorted by `(field_name, reason, owner, date)` byte tuple:
        four str32 values in that order
    tag 0x05:
      u32 intrinsic count
      rows sorted by intrinsic name:
        str32 name, u64 declared cost
    tag 0x06:
      u32 constant count
      rows sorted by qname:
        str32 qname, ConstValueV1
    tag 0x07:
      u32 enum-type count
      types sorted by type name:
        str32 type name, u32 member count,
        str32 members in declaration order
    tag 0x08:
      u8 closed-vocabulary option
      when 1, exactly four EnumKindV1 rows in tag order:
        u8 kind, u8 kind-present option,
        when present, u32 member count and sorted str32 members
    tag 0x09:
      u32 StableElementResolverManifest layout version 1
      32-byte StableElementResolverManifestV1 digest
    tag 0x0a:
      u32 WorldRegisterManifest layout version 1
      32-byte WorldRegisterManifestV1 digest

Vocabulary needs both the outer and per-kind presence bits. No vocabulary is
different from a vocabulary with a present-empty kind: the latter rejects all
members of that kind. Enum type rows sort, but each type's member declaration
order remains semantic. The execution plan, rather than source or file order,
sets rule order. Every other registry or map row uses the declared byte sort.

The TypeEnv exemption ledger appears explicitly. The live rules hash covers
canonical rule ASTs, not that ledger, despite an older source comment. Lint-only
default findings stay out. The rules hash and full canonical AST commit loaded
rule internals; the resolved id vector commits execution order. A semantic
compiler or preparation change requires a new PreparedEnvironment layout
rather than a selective encoding of incidental compiled fields.

The implementation computes this object only after successful preparation. It
recomputes the canonical rules hash from loaded rule forms and compares it with
the rules half of the declared ContentDigest. Formatting and input-file order
cannot move the result.

PreparedEnvironmentDigestV1 is SHA-256 of the exact prepared-environment
bytes.

PreparedEnvironmentV1 is at most 67,108,864 bytes. Its ceilings are 65,536
rules, fields, constants, enum types, and combined stable resolver rows; 64
exemptions and intrinsic rows; 4,096 members in one declared enum; 524,288
members in one vocabulary kind; and 1,048,576 aggregate prepared rows. The
aggregate includes the four vocabulary-kind rows and every vocabulary member.
Encoders enforce the prepared aggregate and combined-byte ceilings separately
from the tick-payload ceilings before allocation.

Content identity remains the existing pair of defines_hash and rules_hash.
Reference identity is one explicit RefDigestV1 containing exactly 32 bytes and
no default. The detached Rust boundary binds both. PER-20 will later validate
and hydrate their durable sources.

## Exact tick payload

TickPayloadV1 is a canonical byte object, not only a digest. It encodes:

    ASCII "babylon.tick-payload" followed by NUL
    u32 layout version 1
    tag 0x01:
      u32 rule-outcome count
      rows in governed execution order:
        str32 rule qname, u64 fired count
    tag 0x02:
      u32 event count
      rows in executable arrival order:
        str32 canonical full EventType/MEMBER
        u32 payload-item count
        rows in live source order, duplicates preserved:
          str32 label symbol, ValueV1
    tag 0x03:
      u32 receipt count
      rows in published AuditReceipt vector order:
        str32 rule qname, RuleRoleV1, EvidenceClassV1,
        u32 ordinal, EffectSignatureV1
    tag 0x04:
      u16 accepted-action-outcome count, exactly zero in Gate 3

The rule-outcome sequence exactly matches PreparedEnvironmentV1 tag 0x03.
TickReport.fired is a checked derived sum of those rows, is not encoded again,
and must agree before publication.

Event order and payload-pair source order are semantic. The live sink accepts
an ordered vector and preserves duplicate labels, so the identity codec does
the same. It does not sort these rows. A bare event member is canonicalized to
`EventType/member`; an already full event stays full. Event rows and
EffectSignatureV1 reuse one BSL canonicalizer.

Reference values resolve through StableElementResolverV1. A dynamically
minted or missing reference refuses; there is no NodeId, HyperedgeId, raw edge
handle, debug representation, unordered JSON, or resolver index in the bytes.

AuditReceipt already excludes identity-bearing graph values and written
values. Its per-rule ordinal remains the published events-first-then-writes
ordinal and is not resorted. StableWorldV1 binds the resulting writes.

TickPayloadV1 is at most 67,108,864 bytes. It permits at most 65,536 rule
tick rows; 1,048,576 items in one event; and 4,096 action outcomes, with the
last count fixed to zero here. The tick aggregate-row and combined-byte
ceilings are distinct from the prepared-environment ceilings. Each local and
aggregate count is checked before
allocation.

TickPayloadDigestV1 is SHA-256 of the exact canonical payload bytes. The
identified report publishes both bytes and digest so PER-20 can verify a
future envelope without reconstructing event data that the tick discarded.

## TickContentHashV1

TickContentHashV1 is SHA-256 of this fixed outer preimage:

    ASCII "babylon.tick-content" followed by NUL
    u32 outer layout version 1
    tag 0x01:
      u32 ReplaySessionId layout version 1
      ReplaySessionIdV1 canonical field
    tag 0x02:
      u64 resolve tick
    tag 0x03:
      u32 ReplaySeed layout version 1
      u32 RNG layout version 2
      ReplaySeed i64 big-endian
    tag 0x04:
      u32 ContentDigest layout version 1
      32-byte defines_hash
      32-byte rules_hash
    tag 0x05:
      u32 RefDigest layout version 1
      32-byte reference digest
    tag 0x06:
      u32 PreparedEnvironment layout version 1
      32-byte prepared-environment digest
    tag 0x07:
      u32 StableWorld layout version 1
      32-byte prior-world digest
    tag 0x08:
      u32 OrderedPracticeActionBatch layout version 1
      32-byte accepted-action-batch digest
    tag 0x09:
      u32 StableWorld layout version 1
      32-byte result-world digest
    tag 0x0a:
      u32 TickPayload layout version 1
      32-byte exact-payload digest

All ten tags are mandatory and appear exactly once in this order. V1 accepts
no extension tags. A new field or register requires a new outer or nested
layout, depending on which contract owns it.

The preimage excludes campaign UUID, PostgreSQL row id, wall time, database
sequence, runtime graph handle, allocator cursor, unordered mapping, P27
bytes, GraphStateHash, and NominalWorldHash.

## Executable replay session seam

The current TickSession API and RNG V1 results remain byte-identical.

PER-60 adds a typed ReplayTickSession. Construction requires:

- ReplaySessionIdV1;
- ReplaySeed;
- RNG layout V2;
- ContentDigest;
- RefDigest;
- scenario, prelude when present, and rule content; and
- one supported graph substrate.

Construction loads PreparedRules, retains the complete stable resolver, checks
the declared rules hash, seals fixed topology, computes exact resolver and
register manifests, and computes PreparedEnvironmentV1. ReplayTickSession
owns the immutable resolver-manifest, register-manifest, and prepared-
environment bytes once. It exposes borrowed diagnostic slices and never clones
those static preimages into a tick report.

TickSession and ReplayTickSession call one internal detached-tick transaction.
They do not duplicate the causal rule loop. A typed execution identity mode
supplies either current RngSeedContext V1 with no authoritative composer or
replay RngSeedContext V2 with the PER-60 composer. Every other preparation,
rule, event, receipt, graph, and publication step is shared.

ReplayTickSession advance accepts an OrderedPracticeActionBatchV1. Gate 3
requires that batch to be empty and to match the replay session and next tick.

One replay advance follows this order:

1. Check and derive the next positive resolve tick.
2. Validate the exact empty OrderedPracticeActionBatchV1.
3. Encode the prior StableGraphStateV1, WorldRegisterSetV1, and StableWorldV1.
4. Run every prepared rule against the detached graph with RNG V2.
5. Encode result StableGraphStateV1, WorldRegisterSetV1, StableWorldV1, and
   exact TickPayloadV1.
6. Compose and hash TickContentHashV1.
7. Reserve the external event sink.
8. Publish graph, events, completed tick, current administrative hashes,
   exact nested identity objects, and TickContentHash as one success.

Every new PER-60 codec or report buffer uses checked size arithmetic and
fallible reservation, and every returned identity or digest completes against
the detached graph before external sink reservation or state publication. Any
returned execution, resolver, topology-seal, codec-reservation, or hash error
leaves the graph, event sink, and completed tick unchanged. Process abort,
including allocator abort, returns no recoverable error and is outside this
atomicity claim. The hash never publishes ahead of the state it identifies.

Prior StableGraphStateV1 bytes are dropped after their digest is complete and
before rule execution. Result stable-graph bytes are dropped after their
digest is complete. The graph encoder exposes exact bytes to contract tests
and explicit diagnostics, but the live report does not retain or clone either
large preimage.

The returned IdentifiedTickReportV1 carries the current TickReport evidence;
exact empty action-batch bytes; exact prior and result register-set and
stable-world bytes; exact tick-payload and outer TickContentHash preimage
bytes; all nested versions and digests; the resolver-manifest and prepared-
environment digests; the prior and result stable-graph digests; and
TickContentHashV1. Static resolver, register-manifest, and prepared bytes stay
borrowed or shared by the session and are not copied per tick. The report is
not a durability claim.

The current Bevy engine link remains on TickSession until Gate 3 gives
it authoritative seed, content, and reference identities. The new replay path
is executable end to end through database-free Rust integration tests.

## Failure rules

PER-60 production code is typed encoder and composer only. ReplayTickSession
does not hydrate any new identity object from raw bytes. PER-20 owns future
production decoders for persisted envelopes and their nested objects.

The production path returns a typed error for:

- an invalid replay session string;
- an unknown numeric seed, RNG, action, prepared-environment, stable-element,
  resolver-manifest, stable-graph, register-manifest, register-set, world,
  payload, or outer layout version supplied to a typed constructor;
- replay seed conversion outside i64;
- a declared rules hash that differs from loaded canonical rules;
- a missing, duplicate, or dangling stable graph identity;
- any topology difference from the sealed resolver;
- an unresolvable event reference;
- a non-finite numeric value;
- a semantic string violation;
- a non-empty runtime action batch;
- a mismatched action session or resolve tick;
- a non-contiguous ordinal or unrecomputed ActionId; or
- any checked count, length, tick, or allocation overflow.

The bounded Rust contract-test parser and independent Python verifier also
refuse an unknown BSL value, type, role, evidence, effect, shape, section, or
kind tag; a noncanonical Boolean or option byte; a truncated field; an
out-of-order or duplicate mandatory tag; and trailing bytes. Those parsers
prove the language-neutral schema but are not production hydration APIs.

No returned failure falls back to alternate hashes, raw runtime ids, debug
strings, JSON, or P27.

## Verification contract

The implementation must add one language-neutral schema and one bounded JSONL
vector corpus. The corpus includes:

- the unchanged RNG V1 four-draw vector;
- ReplaySessionIdV1 minimum and maximum bytes plus space, control, DEL, and
  non-ASCII refusals;
- an asymmetric RNG V2 preimage and digest, the first nine u64 draws so the
  vector crosses the first 64-byte block, and the first f64 bits from a fresh
  stream;
- ReplaySeed i64 minimum, negative one, zero, one, and maximum encodings;
- exact binary stable-element, ASCII carrier-segment, graph-owned final carrier,
  and resolver-manifest bytes and digests;
- one asymmetric ActionId and non-empty ordered-batch codec vector;
- the exact empty runtime action-batch bytes and digest;
- asymmetric stable graph, register manifest, register set, stable world,
  prepared environment, payload, and outer TickContentHash bytes and digests;
- each BSL value, type, role, evidence, effect, shape, vocabulary-presence
  discriminant, and exact UTF-8 governance string;
- nonempty, wrong-session, and wrong-tick outer action-link refusals;
- one operation-specific maximum and maximum-plus-one refusal row for every
  declared semantic string, row, member, aggregate, and byte ceiling; and
- one mutation row for every independently variable identity input and nested
  layout version. Session and tick mutations carry their derived exact-empty
  action-batch links.

Rust contract tests and an independent Python verifier read the same vectors.
Their raw parsers use compiled fixed byte, row, and loop ceilings before they
trust schema fields. Both consumers execute every refusal row. The Rust
production surface remains typed encoder/composer-only.

End-to-end verification must prove:

- two independent processes produce identical canonical bytes and digests;
- MemoryGraph and HypergraphStore produce identical stable bytes;
- genuinely different NodeId and HyperedgeId allocations produce identical
  stable bytes while the current GraphStateHash values differ;
- storage, map, registry, and hyperedge-member insertion order cannot move
  canonical output where order is not semantic;
- changing semantic rule, event, event-payload-pair, receipt, or enum-member
  order moves the owning identity;
- every semantic input mutation moves its owning nested digest and the outer
  TickContentHash;
- ReplaySeed mutation crosses ReplayTickSession, run_tick, DrawContext,
  KernelIntrinsicHost, and a real `rng-draw`, moving the exact written f64
  bits, result StableWorld, and TickContentHash while prepared environment,
  prior world, and the empty batch remain fixed;
- V2 mutations to session, tick, rule domain, subject key, nested node, edge,
  or hyperedge key, and draw slot move the real intrinsic result bits;
- negative zero canonicalizes, while NaN and infinity refuse;
- a failed replay tick publishes no state, event, receipt, or hash;
- a non-empty batch cannot enter ReplayTickSession;
- raw action-batch bytes cannot construct the trusted batch type;
- every count, length, aggregate-row, and canonical-byte ceiling accepts its
  maximum and refuses maximum plus one without partial publication; and
- every existing P27 and RNG V1 vector remains unchanged.

## Ownership

- babylon-kernel owns ReplaySessionIdV1, ReplaySeed, RNG layout and the exact
  ChaCha8 algorithm, low-level seed derivation from validated domain/key bytes,
  digest wrappers, RefDigest, and the outer TickContentHash codec.
- babylon-practice-contract owns ActorOrganizationIdV2, ActionId derivation,
  private ordered-action values, projection validation, and batch bytes.
- babylon-graph owns stable element keys, carrier segments, the private final
  StableCarrierKeyV2 builder, resolver validation and manifest,
  StableGraphStateV1, limits, exact bytes, and hashes.
- babylon-bsl retains scenario node and hyperedge names; owns ValueV1 and the
  BSL type, registry, event, and receipt section codecs; and accepts the typed
  RNG context. It alone assembles the authoritative resolved element stack and
  passes graph-validated carrier bytes to kernel.
- babylon-tick composes PreparedEnvironmentV1, the register manifest and set,
  StableWorldV1, and TickPayloadV1 from owned checked sections. It owns the
  shared transaction, ReplayTickSession, report, and atomic publication.
- babylon-persistence imports the owning kernel digest types directly. It does
  not calculate an independent TickContentHash.

## Explicit non-goals

PER-60 does not add:

- a PostgreSQL table, migration, transaction, or writer;
- CommittedTickEnvelope, Archive outbox, hydration, or cutover;
- a persisted action-batch decoder;
- production raw decoders for the new PER-60 identity objects;
- a raw-byte path that confers accepted-action provenance;
- a campaign UUID inside physics identity;
- player action admission or adjudication;
- BSL intent carriers or practice effects;
- dynamic node or hyperedge identity allocation;
- topology tombstones, generations, journals, or stable-mint rules;
- a Merkle tree or partial-proof protocol;
- a second generic action model; or
- changes to current GraphStateHash, NominalWorldHash, or P27 bytes.

## Supersession and preservation

ADR240 will record this design.

It preserves the Constitution, the current RNG V1 stream, P27 vectors,
GraphStateHash, NominalWorldHash, PracticeIntentV2, PracticeProposalKeyV2,
ResolvedPracticeBatchV2 admission validation, ADR236 order-neutral allocation,
and the Python-writer boundary.

It partially supersedes:

- the historical persistence design and implementation-plan phrases that made
  reproduction of the complete P27 encoder a Rust writer-cutover gate;
- the Python tick-hash module claim that babylon-kernel must produce the
  identical P27 digest;
- the open P27 disposition in the live determinism reference, now resolved as
  frozen reference preservation outside TickContentHashV1;
- the Neel plan and ADR233 destination that placed campaign UUID inside
  NominalWorldHashV2, while preserving campaign-bound input authority and
  durable campaign identity; and
- the ADR235 implication that ResolvedPracticeBatchV2 itself is the final
  detached-tick action identity, while preserving its admission bytes and
  validation role.

ADR220's byte-identity writer gate remains. ADR240 clarifies that it means
all implementations reproduce the accepted language-neutral
TickContentHashV1 vectors; it does not mean those bytes equal P27 JSON.
Campaign identity remains part of persistence, checkpoint, and envelope
identity under PER-20, not material world physics.

ResolvedPracticeBatchV2 remains the validated admission source.
OrderedPracticeActionBatchV1 is its session-scoped identity projection. In
Gate 3, only the empty form is authoritative; future non-empty authority also
requires the accepted-input provenance that PER-60 deliberately does not mint.

<!-- vale write-good.Weasel = YES -->
<!-- vale write-good.TooWordy = YES -->
<!-- vale write-good.ThereIs = YES -->
<!-- vale strunk.CommonlyMisused = YES -->
<!-- vale strunk.ActiveVoice = YES -->
<!-- vale ste.ThisPronoun = YES -->
<!-- vale ste.SentenceLength = YES -->
<!-- vale ste.Semicolon = YES -->
<!-- vale ste.ProcedureLength = YES -->
<!-- vale ste.PassiveVoice = YES -->
<!-- vale ste.OneInstruction = YES -->
<!-- vale ste.Modals = YES -->
<!-- vale ste.LatinAbbrev = YES -->
<!-- vale ste.Gerunds = YES -->
<!-- vale ste.Dictionary = YES -->
<!-- vale ste.Ambiguity = YES -->
<!-- vale Vale.Terms = YES -->
<!-- vale ste.NounClusters = YES -->
<!-- vale ste.UnapprovedWords = YES -->
<!-- vale Vale.Spelling = YES -->
