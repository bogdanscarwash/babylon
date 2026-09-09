//! B0's proof that the client links the engine in-process: run the Slice 1
//! seam (scenario -> rule -> one tick -> state hash) at startup and log it.
//!
//! B2 Task 13 added `EngineSession` — the client's own held, persistent
//! `TickSession`, plus the fips<->`NodeId` map and the tick-0 population
//! baseline the map lenses (`lens.rs`) need. `engine_link_probe` stays
//! exactly as B0 left it (`tests/engine_link.rs`'s pinned-hash test still
//! exercises it); `EngineSession` is a separate surface.
//!
//! **B3 wave-1 Task 5 (plan §2.5/§3.2).** `EngineSession::start` is now
//! GENERIC over any catalog [`crate::story::Story`] rather than the
//! counties-only constructor B2 shipped: the fips<->`NodeId` roster is
//! DERIVED through `crate::story::roster_from_loaded` (never the old
//! hand-transcribed `DEMO_FIPS`, deleted by this task), and the session id
//! is the story's own `session_id` (§3.5), never the retired
//! `"babylon-client-b2-demo"` placeholder. `start_over` (Task 4's narrow
//! seam for the carceral story ahead of this catalog landing) is deleted —
//! absorbed into this wider `start`, exactly as its own doc comment named.
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_kernel::SessionId;
use babylon_tick::{run_once, TickReport, TickSession};

const SCENARIO: &str = include_str!("../../babylon-tick/content/scenarios/two-classes.bscn");
const RULE: &str = include_str!("../../babylon-tick/content/rules/fundamental-theorem.bsl");

/// Run one deterministic tick over the pinned two-classes scenario and
/// fundamental-theorem rule — the same `babylon_tick::run_once` seam the
/// CLI driver uses, so "the client links the engine" means sharing this
/// exact code path, not a lookalike reimplementation.
///
/// # Errors
/// Whatever `babylon_tick::run_once` returns: a scenario load, rule load,
/// or tick-execution failure.
pub fn engine_link_probe() -> Result<TickReport, String> {
    run_once(SCENARIO, RULE)
}

/// The client's own held tick session over the currently-selected
/// [`crate::story::Story`] — `roster` is that story's territory-fips
/// pairs, DERIVED (never hand-transcribed), with admitted rules running
/// every tick in governed phase order. D16 orders same-position ties.
/// `Resource`:
/// `loop_ui::spawn_engine_session_and_hud` inserts one at Startup, held for
/// the whole session; `ui/story_card.rs`'s `N`-key restart replaces it with
/// a fresh one over the catalog's next story.
#[derive(bevy::prelude::Resource)]
pub struct EngineSession {
    pub inner: TickSession<HypergraphStore>,
    pub sink: CollectingSink,
    pub story: &'static crate::story::Story,
    /// This story's territory-fips pairs, derived via
    /// `crate::story::roster_from_loaded`. Naturally empty for a
    /// `map_binding: None` story (zero `TERRITORY` nodes) — §2.11's honest
    /// absence rides on this being empty, not on a separate branch.
    pub roster: Vec<(String, NodeId)>,
    /// The tick-0 population baseline the Population Trend lens measures
    /// against — one entry per `roster` fips. Empty for a story with no map
    /// binding (there is no territory population to baseline).
    pub population_baseline: Vec<(String, f64)>,
    /// Every node this story's scenario minted, of any type — the §2.11
    /// absence banner's own "N nodes, 0 territories" quantity, read once at
    /// `LoadedScenario::node_content_ids.len()`.
    pub node_count: usize,
    /// B3 wave-1 Task 7 (§2.11/`ui::roster_panel`): every `SOCIAL_CLASS`/
    /// `INSTITUTION` node this story's scenario minted, derived via
    /// `crate::story::derive_full_roster` — the selected-node panel's own
    /// roster for a `MapBinding::None` story (no county to click).
    /// Naturally empty for counties (declares neither node type).
    pub full_roster: Vec<(String, NodeId, crate::story::NodeKind)>,
}

impl EngineSession {
    /// Loads `story.scenario_src` TWICE — once here, to recover the
    /// territory roster and the SEEDED (pre-tick) population totals before
    /// `TickSession` takes ownership of its own graph, and once inside
    /// `TickSession::new`. Deterministic loading means both loads mint
    /// identical ids (Task 13's own test proved this for the counties case;
    /// unchanged by this task). This costs one extra scenario parse at
    /// startup (microseconds at this scale) and keeps `TickSession`'s
    /// public surface exactly its own four methods, rather than widening it
    /// to expose its internal graph mutably pre-tick.
    ///
    /// # Errors
    /// The same failure modes `TickSession::new` has (an intrinsic
    /// declaration, a scenario load, or a rule load), plus
    /// `crate::story::roster_from_loaded`'s own (a `TERRITORY` node with no
    /// resolvable content id, or — for a `MapBinding::Fips` story — a
    /// resolved content id that is not a five-digit FIPS).
    ///
    /// # Panics
    /// If `story.session_id` is empty — cannot happen through any
    /// [`crate::story::STORIES`] entry (both are non-empty literals; a
    /// catalog-uniqueness test does not exist for this specifically because
    /// the literal is directly visible at the call site, same posture as
    /// every other `SessionId::new(...).expect(...)` call in this crate).
    pub fn start(story: &'static crate::story::Story) -> Result<Self, String> {
        let mut probe_graph = HypergraphStore::new();
        let loaded = babylon_bsl::scenario::load_scenario(story.scenario_src, &mut probe_graph)
            .map_err(|e| format!("story {:?}: {e}", story.id))?;
        let node_count = loaded.node_content_ids.len();

        let roster =
            crate::story::roster_from_loaded(story, &probe_graph, &loaded.node_content_ids)?;
        let full_roster = crate::story::derive_full_roster(&probe_graph, &loaded.node_content_ids)?;

        // The tick-0 baseline the Population Trend lens measures against —
        // read from THIS graph, before it is discarded, while it still
        // holds only the scenario's seeded (un-ticked) values. A missing
        // attribute fails LOUDLY naming the county and field — a 0.0
        // default here would silently skew every later trend delta
        // (`node_attribute` is "never a default 0.0" for exactly this
        // reason; Copilot review, PR #504). Only meaningful for a story
        // with a map binding — a `None`-binding story has no territories to
        // baseline, so `roster` is already empty and this collects nothing.
        let population_baseline: Vec<(String, f64)> = roster
            .iter()
            .map(|(fips, id)| {
                let read = |field: &str| {
                    probe_graph.node_attribute(*id, field).map_err(|e| {
                        format!("tick-0 baseline: {fips} has no {field}: {}", e.message)
                    })
                };
                let pop_d = read("territory/pop-d")?;
                let pop_p = read("territory/pop-p")?;
                let pop_d_prime = read("territory/pop-d-prime")?;
                Ok((fips.clone(), pop_d + pop_p + pop_d_prime))
            })
            .collect::<Result<_, String>>()?;

        // Concatenation order below is arbitrary. The driver compiles governed
        // phase placement and uses D16 bytes only for same-position ties, so
        // text order has no bearing on execution or the resulting TickReport.
        // babylon-tick's multi_rule_conformance.rs proves this invariance.
        let rule_src = story.rule_srcs.join("\n");
        // §3.5: the story's own scenario qname, transcribed from its
        // `.bscn`'s `(scenario …)` form — never a UUID, never a wall-clock
        // read (III.7). Replaces the retired
        // `SessionId::new("babylon-client-b2-demo")` placeholder.
        let session_id = SessionId::new(story.session_id).expect("catalog ids are non-empty");
        let inner = TickSession::new(
            story.scenario_src,
            &rule_src,
            HypergraphStore::new(),
            session_id,
        )
        .map_err(|e| format!("tick session: {e}"))?;

        Ok(Self {
            inner,
            sink: CollectingSink::default(),
            story,
            roster,
            population_baseline,
            node_count,
            full_roster,
        })
    }

    /// Advances the held session by one tick, against the SAME `sink` every
    /// call — the event feed (`ui/beats.rs`) reads the whole session's
    /// accumulated history.
    ///
    /// # Errors
    /// Whatever `TickSession::advance` returns.
    pub fn advance(&mut self) -> Result<TickReport, String> {
        self.inner.advance(&mut self.sink)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::CountyAtlas;
    use crate::story;

    const ATLAS_BYTES: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../assets/map/county_atlas.bin"
    ));

    #[test]
    fn engine_session_starts_and_the_twelve_fips_resolve_on_the_real_atlas() {
        let session = EngineSession::start(story::counties()).expect("engine session starts");
        assert_eq!(session.roster.len(), 12);
        let atlas = CountyAtlas::parse(ATLAS_BYTES).unwrap();
        for (fips, _id) in &session.roster {
            assert!(
                atlas.index_of_fips(fips).is_some(),
                "demo FIPS {fips} must resolve on the committed atlas"
            );
        }
    }

    #[test]
    fn population_baseline_matches_the_seeded_tick_zero_totals() {
        let session = EngineSession::start(story::counties()).expect("engine session starts");
        assert_eq!(session.population_baseline.len(), 12);
        // Task 7's Step 2 table: index 0 (core x0.95) seeds pop-d=2042,
        // pop-p=5748, pop-d-prime=1710 — total 9,500 exactly. Same fips
        // order as roster (derived, but `graph.nodes` already sorts
        // ascending by NodeId, which is insertion order — the same order
        // the old hand-transcribed DEMO_FIPS assumed).
        let (fips0, total0) = &session.population_baseline[0];
        assert_eq!(fips0, &session.roster[0].0);
        assert!((total0 - 9500.0).abs() < 1e-6, "got {total0}");
    }

    #[test]
    fn engine_session_advance_moves_the_hash_and_runs_both_rules_every_tick() {
        let mut session = EngineSession::start(story::counties()).expect("start");
        let r1 = session.advance().expect("tick 1");
        let r2 = session.advance().expect("tick 2");
        assert_eq!(session.inner.tick(), 2);
        assert_ne!(r1.after, r2.after);
        assert_eq!(r1.per_rule_fired.len(), 2, "both packs run every tick");
    }

    #[test]
    fn engine_session_starts_on_carceral_with_an_empty_roster_and_a_real_node_count() {
        let session = EngineSession::start(story::carceral()).expect("carceral session starts");
        assert!(
            session.roster.is_empty(),
            "carceral mints zero TERRITORY nodes"
        );
        assert!(
            session.population_baseline.is_empty(),
            "no territories to baseline"
        );
        // 5 SOCIAL_CLASS + 1 INSTITUTION, per carceral-arc-conformance.bscn's
        // own header (§2.11's "6 nodes, 0 territories").
        assert_eq!(session.node_count, 6);
    }

    #[test]
    fn carcerals_full_roster_has_five_social_class_nodes_and_one_institution() {
        let session = EngineSession::start(story::carceral()).expect("carceral session starts");
        assert_eq!(session.full_roster.len(), 6);
        let social_class_count = session
            .full_roster
            .iter()
            .filter(|(_, _, kind)| *kind == story::NodeKind::SocialClass)
            .count();
        let institution_count = session
            .full_roster
            .iter()
            .filter(|(_, _, kind)| *kind == story::NodeKind::Institution)
            .count();
        assert_eq!(social_class_count, 5);
        assert_eq!(institution_count, 1);
        // Labels are the scenario's own local names — real, not synthetic.
        assert!(session
            .full_roster
            .iter()
            .any(|(label, _, _)| label == "la-approaching"));
        assert!(session
            .full_roster
            .iter()
            .any(|(label, _, _)| label == "carceral-register"));
    }

    #[test]
    fn counties_full_roster_holds_only_its_six_social_class_fixture_nodes() {
        // us-counties-lifecycle-demo.bscn declares 12 TERRITORY nodes (the
        // real demo counties, excluded from this SOCIAL_CLASS/INSTITUTION
        // roster) PLUS 6 SOCIAL_CLASS fixture nodes (core/bourgeoisie/
        // hermit/last-worker/remnant/dissolved, the vitality demo's own
        // seed) — the derivation is the SAME code path for both stories
        // (`story::derive_full_roster`'s own doc), no branch needed; it
        // simply finds zero INSTITUTION nodes and 6 SOCIAL_CLASS ones here,
        // rather than the empty roster carceral's territory-only
        // counterpart (`roster`) has for THIS story.
        let session = EngineSession::start(story::counties()).expect("counties session starts");
        assert_eq!(session.full_roster.len(), 6);
        assert!(session
            .full_roster
            .iter()
            .all(|(_, _, kind)| *kind == story::NodeKind::SocialClass));
    }
}
