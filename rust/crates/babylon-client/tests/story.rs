//! B3 wave-1 Task 5.1's own RED phase (plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md` §2.5):
//! `STORIES` contains `counties` and `carceral` with unique ids and unique
//! `session_id`s; `Story::by_id("nope")` is an `Err` naming the catalog; the
//! roster is DERIVED (never hand-transcribed); `counties` declares
//! `MapBinding::Fips` and `carceral` declares `None`; every story's
//! `validated_horizon` is positive; every `DeclaredConst` names a source
//! that is a real `file:line`; and the premise-provenance test (I1): each
//! `premise`, normalized, is a substring of its own `scenario_src`,
//! normalized the same way.

use babylon_client::story::{self, MapBinding, Story, STORIES};
use babylon_graph::hypergraph_store::HypergraphStore;
use std::collections::HashSet;

#[test]
fn stories_contains_counties_and_carceral_with_unique_ids_and_session_ids() {
    assert_eq!(STORIES.len(), 2, "wave 1 ships exactly two stories");

    let mut ids = HashSet::new();
    let mut session_ids = HashSet::new();
    for story in STORIES {
        assert!(ids.insert(story.id), "duplicate story id {:?}", story.id);
        assert!(
            session_ids.insert(story.session_id),
            "duplicate session_id {:?}",
            story.session_id
        );
    }
    assert!(ids.contains("counties"));
    assert!(ids.contains("carceral"));
}

#[test]
fn by_id_returns_err_naming_the_catalog_for_an_unknown_id() {
    let err = Story::by_id("nope").expect_err("nope is not in the catalog");
    for story in STORIES {
        assert!(
            err.contains(story.id),
            "error {err:?} must name catalog id {:?}",
            story.id
        );
    }
}

#[test]
fn every_validated_horizon_is_positive() {
    for story in STORIES {
        assert!(
            story.validated_horizon > 0,
            "story {:?} validated_horizon must be positive, got {}",
            story.id,
            story.validated_horizon
        );
    }
}

/// Every `DeclaredConst.source` must cite a real `file:line` (or
/// `file:line-line`) — the file half resolved against
/// `babylon-tick/content/scenarios/`, the same directory every `scenario_src`
/// in this catalog lives in. Task 6 (plan §6.4) fills carceral's three
/// `carceral/*-delay` entries in; counties still declares an EMPTY `delays`
/// slice (a standing fact, not a gap), so this loop stays vacuously true for
/// counties while being exercised for real on carceral's own three rows.
#[test]
fn every_declared_const_names_a_source_that_is_a_real_file_line() {
    let scenarios_dir = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../babylon-tick/content/scenarios"
    ));
    for story in STORIES {
        for dc in story.delays {
            let (file, line_part) = dc.source.split_once(':').unwrap_or_else(|| {
                panic!(
                    "DeclaredConst {:?} source {:?} must be file:line",
                    dc.name, dc.source
                )
            });
            assert!(
                !line_part.is_empty() && line_part.chars().next().unwrap().is_ascii_digit(),
                "DeclaredConst {:?} source {:?} must name a line number after the colon",
                dc.name,
                dc.source
            );
            assert!(
                scenarios_dir.join(file).exists(),
                "DeclaredConst {:?} source {:?} names a file that does not exist under {}",
                dc.name,
                dc.source,
                scenarios_dir.display()
            );
        }
    }
}

/// §2.5 revision 2: the roster is DERIVED from `load_scenario` +
/// `node_content_ids`, never a hand-transcribed list. For every story, every
/// `TERRITORY`/`SOCIAL_CLASS`/`INSTITUTION` node the scenario mints (the
/// only three node types either shipped scenario declares) resolves in
/// `node_content_ids` — proven in both directions: every node of a known
/// type resolves, and the resolved count matches `node_content_ids.len()`
/// exactly (so no node of an UNKNOWN type slipped past this check unnoticed).
#[test]
fn every_story_roster_resolves_every_node_the_scenario_mints() {
    const CANDIDATE_NODE_TYPES: &[&str] = &["TERRITORY", "SOCIAL_CLASS", "INSTITUTION"];
    use babylon_graph::substrate::GraphSubstrate;

    for story in STORIES {
        let mut graph = HypergraphStore::new();
        let loaded = babylon_bsl::scenario::load_scenario(story.scenario_src, &mut graph)
            .unwrap_or_else(|e| panic!("story {:?}: scenario load failed: {e}", story.id));

        let mut total_minted = 0usize;
        for node_type in CANDIDATE_NODE_TYPES {
            for id in graph.nodes(node_type) {
                assert!(
                    loaded.node_content_ids.contains_key(&id),
                    "story {:?}: {node_type} node {id:?} has no resolvable content id",
                    story.id
                );
                total_minted += 1;
            }
        }
        assert_eq!(
            total_minted,
            loaded.node_content_ids.len(),
            "story {:?}: node_content_ids carries {} entries but only {total_minted} nodes \
             resolved across the known node types {CANDIDATE_NODE_TYPES:?} — a node of a type \
             this test does not know about was minted",
            story.id,
            loaded.node_content_ids.len()
        );
    }
}

#[test]
fn counties_declares_fips_and_every_resolved_territory_content_id_is_a_five_digit_fips() {
    let counties = story::counties();
    assert_eq!(counties.map_binding, Some(MapBinding::Fips));

    let roster = story::derive_roster(counties).expect("counties roster derives");
    assert!(
        !roster.is_empty(),
        "counties must derive a non-empty territory roster"
    );
    for (content_id, _) in &roster {
        assert!(
            content_id.len() == 5 && content_id.chars().all(|c| c.is_ascii_digit()),
            "counties territory content id {content_id:?} is not a five-digit FIPS"
        );
    }
}

#[test]
fn carceral_declares_no_map_binding_and_derives_an_empty_roster() {
    let carceral = story::carceral();
    assert_eq!(carceral.map_binding, None);

    let roster = story::derive_roster(carceral).expect("carceral roster derives");
    assert!(
        roster.is_empty(),
        "carceral mints zero TERRITORY nodes — its derived roster must be empty, got {roster:?}"
    );
}

/// I1: strip each line's leading `; `, collapse whitespace, and assert the
/// premise is a substring of the same normalization of its own
/// `scenario_src`. An implementer who "improves" the wording turns this
/// red (§2.5's own stated intent).
fn normalize(text: &str) -> String {
    text.lines()
        .map(|line| line.strip_prefix("; ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn every_premise_normalized_is_a_substring_of_its_own_scenario_source_normalized() {
    for story in STORIES {
        let premise_norm = normalize(story.premise);
        let scenario_norm = normalize(story.scenario_src);
        assert!(
            !premise_norm.is_empty(),
            "story {:?}: premise must not be empty",
            story.id
        );
        assert!(
            scenario_norm.contains(&premise_norm),
            "story {:?}: normalized premise is not a substring of its own scenario_src \
             (premise_source {:?}) — the premise must be TRANSCRIBED, never authored",
            story.id,
            story.premise_source
        );
    }
}
