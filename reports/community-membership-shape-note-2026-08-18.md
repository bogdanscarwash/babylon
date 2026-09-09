# Community's membership-shape requirements against the AG(i) ceremony

**Date:** 2026-08-22 (keeps the plan's dated filename).
**Plan:** `docs/superpowers/plans/2026-08-18-community-port.md` Task 11 — this task writes a design
note and files issues; it implements nothing.
**Audience:** the #653 ceremony's designer (and the Director, for the co-sponsorship call).
**Status:** requirements, stated by the Community port train after its hyperedge lane landed
(#681-#690). Nothing here is a commitment by #653; each row names what Community *needs* and why,
so the ceremony can adjudicate against evidence rather than against a re-read.

## What Community needs, field by field

The frozen `CommunityMembership` payload (`src/babylon/models/entities/community.py:370-417`, the
membership model the port could not carry onto a bare hyperedge) decomposes into three typed fields
plus one override:

| payload | frozen type | this train's requested BSL shape | consumed by (the blocked half) |
|---|---|---|---|
| `role` | `MembershipRole` (5 members, frozen order CORE_ORGANIZER, ACTIVE, PARTICIPANT, PERIPHERAL, SYMPATHIZER — `src/babylon/models/enums/social.py:79-85`) | **int-ordinal**, with `ROLE_STRENGTH_WEIGHTS` (`src/babylon/models/entities/community.py:25-31`: 1.0/0.7/0.4/0.2/0.1) read through a `defconst` ladder | threat scoring (`src/babylon/engine/systems/community.py:579-608`, the role → weight lookup at `:601`) AND the infrastructure maintenance count (`:655-661`, `role == CORE_ORGANIZER`) |
| `strength` | `Coefficient` [0,1] | a `coefficient`-typed payload field | solidarity amplification (`src/babylon/engine/systems/community.py:527-576`, per-endpoint `mem.strength` at `src/babylon/formulas/community.py:111-141`) |
| `visibility` | `Probability` [0,1] | a `probability`-typed payload field | threat scoring (the per-membership `effective_visibility`, `:601`) |
| `overt` | `bool` | the same probability field, with `overt` expressed as `visibility = 1.0` — see the note below | threat scoring |

**On `overt`:** the frozen `effective_visibility`
(`src/babylon/models/entities/community.py:407-417`) is `1.0 if overt else visibility` — a one-bit override on a
[0,1] field. Community's need is the *value*, not the flag: a `visibility` payload seeded 1.0 where
frozen would set `overt = true` carries the same information through the same type. If the ceremony
wants the override's *provenance* (a deliberately-public member vs a legible one), that is a second
field and this train does not need it.

**On the int-ordinal `role`:** the port-estate survey
(`reports/port-estate-survey-2026-08-12.md` finding 6) ruled int-ordinal when D102 made enum fields
unreadable. D102 has since been **discharged** (the P27 territory-port train, Task 1 — `field-of`
over an `:enum-type` field renders `Value::Enum` today). The int-ordinal recommendation stands
anyway, for a different and simpler reason: `role`'s only reads are the two weight/count reads
above, both of which want a ladder they index by ordinal, and a membership-scoped `defenum` would ask AG(i) to
type payload fields *per hyperedge type* with enum rendering — more ceremony than two reads
justify. If the ceremony lands enum-typed payloads for free, the ordinal reading stays valid (the
ordinal IS the index); the ask is the ladder, not the encoding.

## The non-field requirements

1. **Ascending-member-id iteration.** AG(i)'s own obligation (Amendment AG clause (i): payload
   iterates in the member list's ruled order). Community's threat/cost computations sum over a
   member's memberships; iteration order is observable in f64 accumulation, so the ruled order
   must be the iteration order, never an insertion or hash order.
2. **Hash participation.** Payload fields feed the canonical state hash (AG(i) clause
   (i), same clause) — a membership's role/strength/visibility are state, and two worlds differing
   only there must hash differently.
3. **The ceiling axis stays `:max-members`.** Community's census-fed `:max-members` axis (D200)
   already bounds member iteration; attributed membership must not open a second, unbounded
   membership channel beside it.

## One mechanism or two — this train's verdict

ADR198 R4 recorded that "the node-local list-of-structs shape may not even be AG(i)'s shape"
(`ai/decisions/ADR198_program29_substrate_widening_charter.yaml:53-58`), and #653's own body names
the Electoral open-cardinality `allegiance` map (`SocialClass.allegiance: dict[str, float]` keyed by
party org id) as the ceremony's first consumer. This train's verdict, from having built the
hyperedge lane both consumers sit on:

**One mechanism, two payload shapes.** AG(i)'s attributed membership — declared, typed payload
fields on the (member, hyperedge) pair — covers both:

- Community's shape has *fixed arity*: three declared fields on the (class, community) pair.
- the Electoral `allegiance` map is the *same* mechanism one arity down: a (class, party-org)
  membership carrying **one** probability payload is the open-cardinality map, with the party-org
  set enumerated by the hyperedges that exist — "open cardinality" is the hyperedge census's own
  openness, not a second state kind.

What would make this verdict wrong: an Electoral read that needs the allegiance map keyed by
orgs that have *no* hyperedge (a party with zero members must still be addressable). If that
reader exists, the map is node-local map-typed state and the two needs genuinely diverge — named
here so the ceremony checks it deliberately rather than discovering it at the readout.

## What this note does not ask

- No new element kinds, no dyadic-edge landing (AG(i) rejected it; VIII.9 survives verbatim —
  a member list crosses WHOLE, never C(n,2)).
- No relaxation of the fuel/bound disciplines for the new reads; the blocked half's rules will
  declare and measure their bounds like every other rule this train landed.
- Nothing about *writing* memberships mid-tick (the mutation direction stays "whole-hyperedge
  replacement", D26) — Community reads payloads; it never edits them.

## References

- Amendment AG (i), `CONSTITUTION.md` — the ratified mechanism and its obligations.
- ADR198 R4, `ai/decisions/ADR198_program29_substrate_widening_charter.yaml:53-58` — the
  separate-ceremony ruling and its shape caveat.
- The blocked half this unblocks: `docs/superpowers/plans/2026-08-18-community-port.md` §5's
  table (threat scoring, solidarity amplification, infrastructure maintenance).
- The port-estate survey's D102 finding, `reports/port-estate-survey-2026-08-12.md` finding 6
  (historical — D102 discharged since), and the decomposition train's rejection analysis,
  `reports/decomposition-controlratio-bsl-surface-facts-2026-08-17.md:147-176`.
