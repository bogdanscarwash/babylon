//! The story catalog (B3 wave-1 Task 5, plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md`
//! §2.5/§3.1/§3.2/§3.5): `Story`/`StoryArc`/`MapBinding`/`DeclaredConst` +
//! the two-entry `STORIES` catalog + `--story` CLI selection. This is the
//! single derived roster — `EngineSession::start` (`engine_link.rs`)
//! consumes it, `main.rs`'s `--story <id>` flag selects from it, and
//! `ui/story_card.rs` renders it.
//!
//! **The roster is DERIVED, never hand-transcribed (§2.5 revision 2).**
//! [`roster_from_loaded`] reads a story's own `TERRITORY` nodes back out of
//! `babylon_bsl::scenario::LoadedScenario::node_content_ids` — the loader's
//! own local-name -> `NodeId` inversion — rather than a parallel constant
//! array a content edit could silently drift away from (the old
//! `engine_link::DEMO_FIPS`, deleted by this task). For a `MapBinding::Fips`
//! story every resolved content id must parse as a five-digit FIPS; for
//! `carceral` (`map_binding: None`) the scenario mints zero `TERRITORY`
//! nodes, so the SAME code path naturally derives an empty roster — no
//! branch needed to special-case the no-map story (§2.11).

use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use std::collections::HashMap;

/// Which map join a story's roster feeds, if any — `None` (carceral) means
/// "this story has no territorial substrate" (§2.11), not "not yet wired".
/// A single-variant enum today (only `Fips` is a real map join this crate
/// has); the `Option` wrapper, not a second variant, is what spells
/// "no map" — see the module doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapBinding {
    /// The roster's content ids are five-digit FIPS codes joining
    /// `map/bands.rs`'s `atlas.index_of_fips`.
    Fips,
}

/// A story's own known dramatic shape — `last_tick` the furthest tick its
/// content has been proven to reach, `beat_count` how many
/// critical/warning beats its own arc contains. `None` (counties) means
/// "no fixed shape is known" — counties is the ambient world, not a
/// scripted arc; its tick-0 card shows the honest "?" rather than a
/// fabricated total (III.11).
#[derive(Debug, Clone, Copy)]
pub struct StoryArc {
    pub last_tick: i64,
    pub beat_count: usize,
}

/// One coefficient the countdown instrument (Task 6) or the story card
/// needs to name and cite rather than bake in as a bare literal — `source`
/// is a `file:line` or `file:line-line` pointing at the `.bscn`/`defines.yaml`
/// row the value was transcribed from. Task 6's own GREEN deliverable
/// (plan §6.4) transcribes carceral's three delay constants here, each
/// citing its own `defconst` line in `carceral-arc-conformance.bscn`;
/// counties declares an EMPTY `delays` slice — a standing fact ("counties
/// declares no delays," its cadence comes from the per-tick deltas
/// instead, §2.4), not a placeholder.
#[derive(Debug, Clone, Copy)]
pub struct DeclaredConst {
    pub name: &'static str,
    pub value: f64,
    pub source: &'static str,
}

/// One catalog entry — `story.rs`'s own struct (§3.1's module-layout table
/// names this file as where `Story`, `StoryArc`, `MapBinding`,
/// `DeclaredConst`, `STORIES` and CLI selection all live), re-cited
/// verbatim against the merged tree at §3.2.
#[derive(Debug)]
pub struct Story {
    pub id: &'static str,
    pub title: &'static str,
    /// Transcribed from `scenario_src` itself — see [`Self::premise`]'s own
    /// provenance test (`tests/story.rs`'s I1: normalized, it must be a
    /// substring of `scenario_src` normalized the same way). Never authored
    /// prose; Amendment AD reserves that to the Director (§2.5's own I1).
    pub premise: &'static str,
    pub premise_source: &'static str,
    pub scenario_src: &'static str,
    pub rule_srcs: &'static [&'static str],
    /// D179/§3.5: the scenario's own qualified name, transcribed from its
    /// `.bscn`'s `(scenario …)` form — never a UUID, never a wall-clock read
    /// (III.7). Closes ADR213 follow-on (iii).
    pub session_id: &'static str,
    pub map_binding: Option<MapBinding>,
    pub arc: Option<StoryArc>,
    pub validated_horizon: i64,
    pub delays: &'static [DeclaredConst],
}

impl Story {
    /// Looks `id` up in [`STORIES`] — `Err` names every catalog id so a
    /// typo's error message IS the fix.
    ///
    /// # Errors
    /// `id` matches no [`STORIES`] entry.
    pub fn by_id(id: &str) -> Result<&'static Story, String> {
        STORIES.iter().find(|s| s.id == id).ok_or_else(|| {
            let known: Vec<&str> = STORIES.iter().map(|s| s.id).collect();
            format!(
                "no story named {id:?} — the catalog has: {}",
                known.join(", ")
            )
        })
    }
}

/// Convenience accessor for the counties story — every headless test in
/// this crate that needs "the counties story" explicitly reaches for this
/// rather than indexing `STORIES` by position (`STORIES[0]` is an
/// implementation detail; `counties()` is the name).
///
/// # Panics
/// If `STORIES` ever stops declaring a `"counties"` entry — cannot happen
/// through any change this crate's own tests would let land
/// (`tests/story.rs`'s own `stories_contains_counties_and_carceral_...`).
#[must_use]
pub fn counties() -> &'static Story {
    Story::by_id("counties").expect("counties is always in STORIES")
}

/// Convenience accessor for the carceral story — see [`counties`].
///
/// # Panics
/// If `STORIES` ever stops declaring a `"carceral"` entry — see
/// [`counties`]'s own doc.
#[must_use]
pub fn carceral() -> &'static Story {
    Story::by_id("carceral").expect("carceral is always in STORIES")
}

/// The catalog entry that follows `current`, cyclically — the `N`-key
/// restart's own "next" (`ui/story_card.rs`, I8). Identity comparison
/// (`std::ptr::eq`) rather than `id ==`: every `&'static Story` this crate
/// ever hands a caller points INTO this slice, so pointer identity is exact
/// and needs no `PartialEq` derive on `Story` itself.
///
/// # Panics
/// If `current` is not a `STORIES` element — cannot happen through any call
/// site this crate has.
#[must_use]
pub fn next_story(current: &'static Story) -> &'static Story {
    let idx = STORIES
        .iter()
        .position(|s| std::ptr::eq(s, current))
        .expect("current story must be a STORIES element");
    &STORIES[(idx + 1) % STORIES.len()]
}

/// Parses `--story <id>` out of a raw argument slice (already stripped of
/// `argv[0]`) — bare `std::env::args()`, matching `babylon-tick/src/main.rs`'s
/// own precedent (no `clap` dependency exists in this workspace). No
/// `--story` flag at all resolves to [`STORIES`]`[0]` (counties) — the
/// default-experience decision (§2.5 I8: the county atlas is the crate's
/// only spatial instrument, so a first-run viewer who types nothing still
/// sees the map breathe).
///
/// # Errors
/// `--story` with no following value, or an id [`Story::by_id`] does not
/// recognize.
pub fn select_story(args: &[String]) -> Result<&'static Story, String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--story" {
            let id = iter
                .next()
                .ok_or_else(|| "--story requires a value".to_owned())?;
            return Story::by_id(id);
        }
    }
    Ok(&STORIES[0])
}

/// The story `main.rs`'s `--story` flag (or a test's own explicit choice)
/// selected — inserted BEFORE `Startup` (§2.5 Minor 7): `EngineSession::start`
/// runs inside `TickLoopPlugin`'s own Startup system and took no inputs of
/// its own before this task. No `Default` impl on purpose, matching
/// `lens::CurrentLensData`'s own "nothing may construct this half-built"
/// discipline — every app-builder, production or test, must say which
/// story it wants; a missing resource panics loudly at Startup rather than
/// silently defaulting.
#[derive(bevy::prelude::Resource, Clone, Copy)]
pub struct SelectedStory(pub &'static Story);

fn is_five_digit_fips(s: &str) -> bool {
    s.len() == 5 && s.bytes().all(|b| b.is_ascii_digit())
}

/// `us-counties-lifecycle-demo.bscn`'s own local-name convention prefixes
/// every territory's bare FIPS with `"county-"` for readability in the
/// `.bscn` source itself (`(node county-01001 NodeType/TERRITORY ...)`,
/// verified directly against the file — the content id is NOT the bare
/// FIPS by itself). Stripping that convention-specific prefix recovers the
/// same bare digit string the old hand-transcribed `DEMO_FIPS` array held
/// (`"01001"`, never `"county-01001"`) — a no-op via `unwrap_or` for any
/// content id that does not carry the prefix, so this is safe to apply
/// unconditionally rather than gating it on the story.
fn fips_from_content_id(content_id: &str) -> &str {
    content_id.strip_prefix("county-").unwrap_or(content_id)
}

/// The shared derivation core (§2.5/§3.2): walks `graph`'s own `TERRITORY`
/// nodes, resolves each through `node_content_ids` (a wiring bug, named
/// loudly, if one is missing — `load_scenario`'s own local-name discipline
/// makes this unreachable in practice, but the check costs nothing), strips
/// the scenario's own `"county-"` local-name prefix (see
/// `fips_from_content_id`), and for a `MapBinding::Fips` story asserts
/// every resolved (stripped) content id is a five-digit FIPS. Carceral's
/// own zero `TERRITORY` nodes make this naturally return an empty roster —
/// the SAME path §2.11's honest absence rides on, not a separate one.
///
/// `EngineSession::start` (`engine_link.rs`) calls this directly against
/// its own probe graph (it already loads the scenario once for the tick-0
/// population baseline); [`derive_roster`] below is the test/tooling
/// convenience that does its own throwaway load first.
///
/// # Errors
/// A `TERRITORY` node with no resolvable content id (a wiring bug), or (for
/// a `Fips` binding) a resolved (stripped) content id that is not a
/// five-digit FIPS — named, never silently dropped or coerced.
// `node_content_ids` has exactly ONE producer in this whole workspace —
// `babylon_bsl::scenario::LoadedScenario::node_content_ids`, declared
// `HashMap<NodeId, String>` (the default `RandomState` hasher) in that
// crate itself — so generalizing this parameter over an arbitrary
// `BuildHasher` would be ceremony with no real caller, never a genuine
// second hasher this signature needs to admit.
#[allow(clippy::implicit_hasher)]
pub fn roster_from_loaded(
    story: &Story,
    graph: &dyn GraphSubstrate,
    node_content_ids: &HashMap<NodeId, String>,
) -> Result<Vec<(String, NodeId)>, String> {
    let mut roster = Vec::new();
    let mut bad_fips = Vec::new();
    for id in graph.nodes("TERRITORY") {
        let content_id = node_content_ids.get(&id).ok_or_else(|| {
            format!(
                "story {:?}: TERRITORY node {id:?} minted with no resolvable local name \
                 (node_content_ids is incomplete — a load_scenario bug, not a story-content one)",
                story.id
            )
        })?;
        let fips = fips_from_content_id(content_id);
        if story.map_binding == Some(MapBinding::Fips) && !is_five_digit_fips(fips) {
            bad_fips.push(content_id.clone());
        }
        roster.push((fips.to_owned(), id));
    }
    if !bad_fips.is_empty() {
        return Err(format!(
            "story {:?} declares MapBinding::Fips but these resolved territory content ids are \
             not five-digit FIPS: {bad_fips:?}",
            story.id
        ));
    }
    roster.sort_by_key(|(_, id)| *id);
    Ok(roster)
}

/// Test/tooling convenience: loads `story.scenario_src` into a fresh
/// throwaway graph and derives its roster in one call. `EngineSession::start`
/// does NOT call this — it already needs its own loaded probe graph for the
/// tick-0 population baseline, so it calls [`roster_from_loaded`] directly
/// against that graph instead, avoiding a third scenario parse per session
/// start.
///
/// # Errors
/// The scenario fails to load, or [`roster_from_loaded`]'s own errors.
pub fn derive_roster(story: &Story) -> Result<Vec<(String, NodeId)>, String> {
    let mut graph = HypergraphStore::new();
    let loaded = babylon_bsl::scenario::load_scenario(story.scenario_src, &mut graph)
        .map_err(|e| format!("story {:?}: scenario load failed: {e}", story.id))?;
    roster_from_loaded(story, &graph, &loaded.node_content_ids)
}

/// Which of `ui::roster_panel`'s two published-field tables a
/// [`derive_full_roster`] entry's own node reads through — carceral's own
/// two node types (`carceral-arc-conformance.bscn`'s own header:
/// `(defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))`). A third variant
/// is added the day a THIRD `MapBinding::None` story ships with a THIRD
/// node type; today only carceral has one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// A `social-class/*` node.
    SocialClass,
    /// The `institution/*` carrier.
    Institution,
}

/// Every `SOCIAL_CLASS`/`INSTITUTION` node a story's scenario minted,
/// labeled with its own content id (`node_content_ids`) and tagged with
/// which published-field table it reads through — the no-map counterpart
/// to [`roster_from_loaded`]'s territory-only roster (§2.11: a
/// `MapBinding::None` story has no county to click, so the selected-node
/// panel walks THIS roster by `\u{2191}`/`\u{2193}` instead,
/// `ui::roster_panel`). For a `MapBinding::Fips` story (counties declares
/// neither node type) this naturally returns an empty roster — the SAME
/// "no branch needed" shape [`roster_from_loaded`]'s own doc already
/// establishes for the county-map case, applied here to its complement.
///
/// # Errors
/// A `SOCIAL_CLASS`/`INSTITUTION` node minted with no resolvable content
/// id — a `load_scenario` wiring bug, unreachable in practice for either
/// shipped story, named loudly rather than silently dropped.
// See `roster_from_loaded`'s own identical exemption comment: exactly one
// producer of `HashMap<NodeId, String>` exists in this workspace, so
// generalizing over `BuildHasher` here would be ceremony with no real
// caller.
#[allow(clippy::implicit_hasher)]
pub fn derive_full_roster(
    graph: &dyn GraphSubstrate,
    node_content_ids: &HashMap<NodeId, String>,
) -> Result<Vec<(String, NodeId, NodeKind)>, String> {
    let mut roster = Vec::new();
    for (type_str, kind) in [
        ("SOCIAL_CLASS", NodeKind::SocialClass),
        ("INSTITUTION", NodeKind::Institution),
    ] {
        for id in graph.nodes(type_str) {
            let label = node_content_ids.get(&id).ok_or_else(|| {
                format!("node {id:?} (type {type_str}) minted with no resolvable local name")
            })?;
            roster.push((label.clone(), id, kind));
        }
    }
    roster.sort_by_key(|(_, id, _)| *id);
    Ok(roster)
}

const COUNTIES_SCENARIO: &str =
    include_str!("../../babylon-tick/content/scenarios/us-counties-lifecycle-demo.bscn");
const COUNTIES_VITALITY: &str = include_str!("../../babylon-tick/content/rules/vitality.bsl");
const COUNTIES_LIFECYCLE: &str = include_str!("../../babylon-tick/content/rules/lifecycle.bsl");
const COUNTIES_RULES: &[&str] = &[COUNTIES_VITALITY, COUNTIES_LIFECYCLE];

/// Transcribed verbatim from `us-counties-lifecycle-demo.bscn:1-5` — the
/// premise-provenance test (`tests/story.rs`) proves this stays a substring
/// of [`COUNTIES_SCENARIO`], normalized.
const COUNTIES_PREMISE: &str = "\
; The Program 28 B2 demo world (Phase B, Task 7): twelve real-FIPS counties
; carrying the lifecycle/dpd-circuit rule pack, plus the six
; vitality/subsistence-and-death fixture nodes verbatim — one scenario,
; two node types, both rule packs running together in governed phase order:
; Vitality @1 before Lifecycle @7.";

const CARCERAL_SCENARIO: &str =
    include_str!("../../babylon-tick/content/scenarios/carceral-arc-conformance.bscn");
const CARCERAL_DECOMPOSITION: &str =
    include_str!("../../babylon-tick/content/rules/decomposition.bsl");
const CARCERAL_CONTROL_RATIO: &str =
    include_str!("../../babylon-tick/content/rules/control-ratio.bsl");
const CARCERAL_RULES: &[&str] = &[CARCERAL_DECOMPOSITION, CARCERAL_CONTROL_RATIO];

/// Transcribed verbatim from `carceral-arc-conformance.bscn:11-22` (the
/// DERIVED TICK SCHEDULE block and its outcome sentence — §2.5 I1) —
/// through line 22, the last content line; line 23 is a bare `;` paragraph
/// separator contributing no text, so it is not part of this literal, even
/// though the citation names the same 11-23 span the plan's own §2.5 does.
const CARCERAL_PREMISE: &str = "\
; **DERIVED TICK SCHEDULE (verified against the frozen mirror below, NOT
; trusted from the plan's own illustrative numbers, per this task's own
; brief):**
;   SUPERWAGE_CRISIS      tick 1   — la-approaching's wealth (515) clears
;                                     the \"approaching\" gate at tick 1.
;   CLASS_DECOMPOSITION   tick 53  — 1 + carceral/decomposition-delay (52).
;   CONTROL_RATIO_CRISIS  tick 105 — 53 + carceral/control-ratio-delay (52).
;   TERMINAL_DECISION     tick 106 — 105 + carceral/terminal-decision-delay (1).
; Outcome: GENOCIDE (avg-organization ~= 0.0563 << revolution-threshold 0.5
; — ip-seed's organization is UNTOUCHED by decomposition's intake, p04/p05
; never write it, so the 510 newly-active prisoners carry organization 0.0;
; only lumpen's pre-existing 200 @ 0.2 contribute anything nonzero).";

/// The two shipped stories (§2.5/§0.4) — `counties` first and default (I8:
/// the county atlas is the crate's only spatial instrument, so a first-run
/// viewer who types nothing still sees the map breathe); `carceral` one
/// keystroke (`--story carceral`, or `N` at runtime) away.
///
/// **`arc`/`delays`.** `carceral.arc` names the four already-proven beats
/// (ticks 1/53/105/106, `tests/autopause.rs`'s own `B`-press sequence) —
/// transcribed fact, not new math. `counties.arc` is `None`: counties is the
/// ambient world, not a scripted arc, so no fixed beat total is honest to
/// claim. `carceral.delays` names its own three `carceral/*-delay`
/// defconsts (Task 6, plan §6.4), each citing its own
/// `carceral-arc-conformance.bscn` line — `ui::countdown`'s own delay-chain
/// table is what pairs each one with the live engine field it counts down
/// from. `counties.delays` stays EMPTY — "counties declares no delays" is a
/// standing fact (its cadence comes from the per-tick deltas instead,
/// §2.4), not a gap.
///
/// **`static`, never `const` (load-bearing).** [`next_story`] and
/// `ui/story_card.rs`'s own `catalog_row` both use `std::ptr::eq` to ask
/// "is this catalog entry the currently-selected one" — a `const` item does
/// not guarantee
/// ONE stable address (each referencing codegen unit may receive its own
/// promoted copy of the array; observed directly: `next_story`'s own test
/// passed by accident of codegen-unit placement while
/// `format_story_card`'s catalog-marker test failed cross-CGU with `const`).
/// `static` is the one Rust item kind with a language-guaranteed single
/// address for its whole `'static` lifetime — reverting this to `const`
/// silently reintroduces that flakiness.
pub static STORIES: &[Story] = &[
    Story {
        id: "counties",
        title: "The County Atlas",
        premise: COUNTIES_PREMISE,
        premise_source: "us-counties-lifecycle-demo.bscn:1-5",
        scenario_src: COUNTIES_SCENARIO,
        rule_srcs: COUNTIES_RULES,
        session_id: "lifecycle/us-counties-demo",
        map_binding: Some(MapBinding::Fips),
        arc: None,
        validated_horizon: 600, // measured, tests/beats.rs::counties_stay_numerically_sane_to_the_validated_horizon (Task 4)
        delays: &[],
    },
    Story {
        id: "carceral",
        title: "The Carceral Arc",
        premise: CARCERAL_PREMISE,
        premise_source: "carceral-arc-conformance.bscn:11-23",
        scenario_src: CARCERAL_SCENARIO,
        rule_srcs: CARCERAL_RULES,
        session_id: "carceral/arc-conformance",
        map_binding: None,
        arc: Some(StoryArc {
            last_tick: 106,
            beat_count: 4,
        }),
        // babylon-tick/tests/carceral_arc_conformance.rs:146's own LAST_TICK
        // — "comfortably past the derived TERMINAL_DECISION tick (106), with
        // margin to prove nothing fires a fifth time afterward."
        validated_horizon: 110,
        // Task 6 (plan §6.4): the three `carceral/*-delay` defconsts,
        // transcribed verbatim from their own `carceral-arc-conformance.bscn`
        // `defconst` lines (`content/scenarios/carceral-arc-conformance.bscn:
        // 137-139`) — `ui::countdown`'s own delay-chain table pairs each
        // name with the live `institution/*` field it counts down from.
        delays: &[
            DeclaredConst {
                name: "carceral/decomposition-delay",
                value: 52.0,
                source: "carceral-arc-conformance.bscn:137",
            },
            DeclaredConst {
                name: "carceral/control-ratio-delay",
                value: 52.0,
                source: "carceral-arc-conformance.bscn:138",
            },
            DeclaredConst {
                name: "carceral/terminal-decision-delay",
                value: 1.0,
                source: "carceral-arc-conformance.bscn:139",
            },
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_story_with_no_flag_defaults_to_counties() {
        let selected = select_story(&[]).expect("no flag resolves");
        assert_eq!(selected.id, "counties");
    }

    #[test]
    fn select_story_with_a_known_flag_resolves_it() {
        let args: Vec<String> = vec!["--story".to_owned(), "carceral".to_owned()];
        let selected = select_story(&args).expect("carceral resolves");
        assert_eq!(selected.id, "carceral");
    }

    #[test]
    fn select_story_with_an_unknown_id_is_an_err_naming_the_catalog() {
        let args: Vec<String> = vec!["--story".to_owned(), "nope".to_owned()];
        let err = select_story(&args).expect_err("nope is not in the catalog");
        assert!(err.contains("counties"));
        assert!(err.contains("carceral"));
    }

    #[test]
    fn select_story_with_a_dangling_flag_is_an_err() {
        let args: Vec<String> = vec!["--story".to_owned()];
        assert!(select_story(&args).is_err());
    }

    #[test]
    fn next_story_cycles_and_wraps() {
        let counties = counties();
        let carceral = carceral();
        assert!(std::ptr::eq(next_story(counties), carceral));
        assert!(std::ptr::eq(next_story(carceral), counties));
    }
}
