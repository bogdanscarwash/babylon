//! The selected-node panel's own no-map path (B3 wave-1 Task 7.5, plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md` §2.11,
//! task-7-brief.md): for a `MapBinding::None` story there is no county to
//! click, so [`SelectedRosterIndex`] + [`cycle_selected_roster_index`] walk
//! `EngineSession::full_roster` by `\u{2191}`/`\u{2193}` instead, and
//! [`format_roster_panel`] renders the selected node's own published
//! fields through [`crate::projection::Projector`] — the SAME honest-
//! Provenance seam every other panel in this crate reads through
//! (§2.6). `loop_ui::refresh_state_panel` is the one caller: it renders
//! THIS module's output in place of the `SelectedCounty` path whenever
//! `session.story.map_binding` is `None`, both sharing the one
//! `StatePanelText` entity rather than spawning a second panel.
//!
//! **The published-field tables are story-specific, like
//! `narration::NARRATION_TABLE`'s own per-`EventType` hardcoding and
//! `ui::countdown::CARCERAL_STEPS`'s own per-story delay chain** — only
//! carceral has a no-map roster today; a second `MapBinding::None` story
//! adds its own row to `SOCIAL_CLASS_FIELDS`/`INSTITUTION_FIELDS` (below,
//! private module constants — not public doc links) rather than
//! generalizing a table nobody else has read yet.
//!
//! **The seeded-0 trap (§2.4).** Two `institution/*-tick` fields
//! (`decomposition-fire-tick`, `control-crisis-tick`) are seeded literal
//! `0` and only overwritten when their owning rule fires — publishing them
//! through a bare `Projector::read` would fabricate "fired at tick 0" for
//! every earlier tick. `InstitutionField::ready_field` (below, a private
//! struct field — not a public doc link) routes those two through
//! `crate::ui::countdown::read_gated_operand` (also private — the SAME
//! gate `ui::countdown::resolve` already applies to the identical fields,
//! reused here, never duplicated — task-7-review.md's own MAJOR finding).

use crate::engine_link::EngineSession;
use crate::projection::Projector;
use crate::story::NodeKind;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use bevy::prelude::*;

/// The carceral world's own `social-class/*` fields (task-7-brief.md's own
/// literal list): population, wealth, organization, active.
const SOCIAL_CLASS_FIELDS: &[&str] = &[
    "social-class/population",
    "social-class/wealth",
    "social-class/organization",
    "social-class/active",
];

/// One row of the carrier's own published-field table. `ready_field` is
/// `Some(...)` for the two `institution/*-tick` latch OPERANDS that hit the
/// seeded-0 trap (§2.4, task-7-review.md's own MAJOR finding):
/// `institution/decomposition-fire-tick` and `institution/control-crisis-tick`
/// are seeded literal `0` in `carceral-arc-conformance.bscn` and only
/// overwritten when their owning rule fires
/// (`decomposition.bsl:309`/`control-ratio.bsl:338`, tick 53/105) — reading
/// either through a bare `Projector::material()` before then would render a
/// fabricated "fired at tick 0". `None` for every other field: a census
/// count or a 0/1 flag genuinely honest at a seeded 0, never a tick marker.
struct InstitutionField {
    field: &'static str,
    /// The `institution/*` flag that must read `1` before `field` is
    /// trusted — matches `ui::countdown::CARCERAL_STEPS`'s own
    /// `ready_field` for the SAME operand, and is read through the SAME
    /// [`crate::ui::countdown::read_gated_operand`] this module calls
    /// rather than a second hand-rolled gate.
    ready_field: Option<&'static str>,
}

/// The carrier's own `institution/*` fields — the task brief's own three
/// named examples (enforcer-population, prisoner-population,
/// decomposition-fire-tick) plus the remaining census/latch fields that
/// give the whole arc's own progress an honest, complete picture (never an
/// arbitrarily narrower cut than what the brief's own "…" already signals
/// exists).
const INSTITUTION_FIELDS: &[InstitutionField] = &[
    InstitutionField {
        field: "institution/enforcer-population",
        ready_field: None,
    },
    InstitutionField {
        field: "institution/prisoner-population",
        ready_field: None,
    },
    InstitutionField {
        field: "institution/decomposition-fire-tick",
        ready_field: Some("institution/decomposition-fired-known"),
    },
    InstitutionField {
        field: "institution/prisoner-org-weighted",
        ready_field: None,
    },
    InstitutionField {
        field: "institution/superwage-crisis-known",
        ready_field: None,
    },
    InstitutionField {
        field: "institution/decomposition-fired-known",
        ready_field: None,
    },
    InstitutionField {
        field: "institution/control-crisis-tick",
        ready_field: Some("institution/control-crisis-emitted"),
    },
    InstitutionField {
        field: "institution/control-crisis-emitted",
        ready_field: None,
    },
    InstitutionField {
        field: "institution/terminal-decision-emitted",
        ready_field: None,
    },
];

/// The `\u{2191}`/`\u{2193}`-selected index into `EngineSession::full_roster` —
/// `None` before the first arrow press (the panel renders nothing until a
/// player actually asks for a node, matching this crate's own "recallable,
/// not forced open" precedent). No `Default` derive that would silently
/// pick index `0`: an unselected roster and a roster whose first element is
/// selected are genuinely different states, and this resource must be able
/// to represent both.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct SelectedRosterIndex(pub Option<usize>);

/// `Update` system: `\u{2191}`/`\u{2193}` cycle [`SelectedRosterIndex`]
/// through `session.full_roster`, wrapping at both ends. A no-op for a
/// `MapBinding::Fips` story (counties) or an empty roster — the county map
/// already owns arrow-key-free click selection there, and this system must
/// never fight it for the same keys.
pub fn cycle_selected_roster_index(
    keys: Res<ButtonInput<KeyCode>>,
    session: Res<EngineSession>,
    mut selected: ResMut<SelectedRosterIndex>,
) {
    if session.story.map_binding.is_some() || session.full_roster.is_empty() {
        return;
    }
    let len = session.full_roster.len();
    if keys.just_pressed(KeyCode::ArrowDown) {
        selected.0 = Some(selected.0.map_or(0, |i| (i + 1) % len));
    } else if keys.just_pressed(KeyCode::ArrowUp) {
        selected.0 = Some(selected.0.map_or(0, |i| (i + len - 1) % len));
    }
}

/// Renders one roster field through the projector at 2 decimal places —
/// `Material` shows the live numeral, every other `Provenance` shows its
/// declared reason with no digit (`projection::Reading::render`'s own
/// contract). `ready_field: Some(...)` routes the read through
/// [`crate::ui::countdown::read_gated_operand`] — the SAME seeded-0-trap
/// gate `ui::countdown::resolve` applies to these exact fields, reused
/// here rather than reimplemented; `None` is an ordinary, ungated
/// `Projector::read`.
fn format_field_line(
    projector: &Projector,
    graph: &dyn GraphSubstrate,
    id: NodeId,
    field: &str,
    ready_field: Option<&str>,
) -> String {
    let reading = match ready_field {
        Some(ready) => crate::ui::countdown::read_gated_operand(projector, graph, id, ready, field),
        None => projector.read(graph, id, field),
    };
    format!("  {field}: {}", reading.render(2))
}

/// Renders the selected-node panel's no-map path: the roster position
/// (`label (N/total)`), then every one of `kind`'s own published fields,
/// each through [`crate::projection::Projector`]. Empty string when
/// nothing is selected yet, or the selected index has fallen out of range
/// (cannot happen through [`cycle_selected_roster_index`] alone, but a
/// fresh `N`-restart resets the resource to `None` rather than leaving a
/// stale index — this is the second line of defense, not the only one).
#[must_use]
pub fn format_roster_panel(
    graph: &dyn GraphSubstrate,
    roster: &[(String, NodeId, NodeKind)],
    selected: Option<usize>,
) -> String {
    let Some(idx) = selected else {
        return String::new();
    };
    let Some((label, id, kind)) = roster.get(idx) else {
        return String::new();
    };
    let projector = Projector::material();
    let mut lines = vec![format!("{label} ({}/{})", idx + 1, roster.len())];
    match kind {
        NodeKind::SocialClass => lines.extend(
            SOCIAL_CLASS_FIELDS
                .iter()
                .map(|field| format_field_line(&projector, graph, *id, field, None)),
        ),
        NodeKind::Institution => lines.extend(
            INSTITUTION_FIELDS
                .iter()
                .map(|f| format_field_line(&projector, graph, *id, f.field, f.ready_field)),
        ),
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_graph::hypergraph_store::HypergraphStore;

    fn social_class_node(graph: &mut HypergraphStore) -> NodeId {
        let id = graph.add_node("SOCIAL_CLASS").expect("add social-class");
        graph
            .update_node(id, "social-class/population", 600.0)
            .expect("population");
        graph
            .update_node(id, "social-class/wealth", 515.0)
            .expect("wealth");
        graph
            .update_node(id, "social-class/organization", 0.2)
            .expect("organization");
        graph
            .update_node(id, "social-class/active", 1.0)
            .expect("active");
        id
    }

    #[test]
    fn nothing_selected_renders_empty() {
        let graph = HypergraphStore::new();
        assert_eq!(format_roster_panel(&graph, &[], None), "");
    }

    #[test]
    fn an_out_of_range_index_renders_empty() {
        let mut graph = HypergraphStore::new();
        let id = social_class_node(&mut graph);
        let roster = vec![("la-approaching".to_owned(), id, NodeKind::SocialClass)];
        assert_eq!(format_roster_panel(&graph, &roster, Some(5)), "");
    }

    #[test]
    fn a_social_class_selection_renders_its_four_published_fields() {
        let mut graph = HypergraphStore::new();
        let id = social_class_node(&mut graph);
        let roster = vec![("la-approaching".to_owned(), id, NodeKind::SocialClass)];
        let rendered = format_roster_panel(&graph, &roster, Some(0));
        assert!(
            rendered.starts_with("la-approaching (1/1)"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("social-class/population: 600.00"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("social-class/wealth: 515.00"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("social-class/organization: 0.20"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("social-class/active: 1.00"),
            "got {rendered:?}"
        );
    }

    /// Seeds every `institution/*` field the real `carceral-arc-conformance.bscn`
    /// carrier seeds, at its own real seeded values — `decomposition-fire-tick`/
    /// `control-crisis-tick` are seeded literal `0`, exactly as production, NOT
    /// left unwritten (the fixture-shape defect class CLAUDE.md names: a
    /// fixture stamping a shape production never emits gives a green test over
    /// a dead code path — here, a fixture that reads back `Absent` instead of
    /// the real `Material(0.0)` the seeded scenario actually produces).
    fn seeded_carrier_node(graph: &mut HypergraphStore) -> NodeId {
        let id = graph.add_node("INSTITUTION").expect("add institution");
        graph
            .update_node(id, "institution/enforcer-population", 110.0)
            .expect("enforcer-population");
        graph
            .update_node(id, "institution/prisoner-population", 710.0)
            .expect("prisoner-population");
        graph
            .update_node(id, "institution/decomposition-fire-tick", 0.0)
            .expect("decomposition-fire-tick seeded 0");
        graph
            .update_node(id, "institution/decomposition-fired-known", 0.0)
            .expect("decomposition-fired-known seeded 0");
        graph
            .update_node(id, "institution/control-crisis-tick", 0.0)
            .expect("control-crisis-tick seeded 0");
        graph
            .update_node(id, "institution/control-crisis-emitted", 0.0)
            .expect("control-crisis-emitted seeded 0");
        id
    }

    #[test]
    fn an_institution_selection_renders_the_carrier_fields_through_the_projector() {
        let mut graph = HypergraphStore::new();
        let id = seeded_carrier_node(&mut graph);
        let roster = vec![("carceral-register".to_owned(), id, NodeKind::Institution)];
        let rendered = format_roster_panel(&graph, &roster, Some(0));
        assert!(
            rendered.contains("institution/enforcer-population: 110.00"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("institution/prisoner-population: 710.00"),
            "got {rendered:?}"
        );
        // The seeded-0 trap (§2.4): both latches read literal 0 here (their
        // own owning rule has not fired), which is EXACTLY the shape a real
        // fresh carceral session has at tick 0 — the panel must render the
        // honest not-yet-latched reason, never the seeded 0 itself (which
        // would fabricate "fired at tick 0").
        assert!(
            rendered.contains("institution/decomposition-fire-tick: not computed by this port"),
            "a seeded-0, not-yet-fired tick field must render the honest NotComputed reason, \
             never the literal 0 it holds, got {rendered:?}"
        );
        assert!(
            !rendered.contains("institution/decomposition-fire-tick: 0.00"),
            "must never render the seeded 0 as a fabricated fired-at-tick-0 claim, \
             got {rendered:?}"
        );
        assert!(
            rendered.contains("institution/control-crisis-tick: not computed by this port"),
            "got {rendered:?}"
        );
        assert!(
            !rendered.contains("institution/control-crisis-tick: 0.00"),
            "got {rendered:?}"
        );
    }

    #[test]
    fn an_institution_selection_renders_the_real_tick_once_its_latch_flips() {
        let mut graph = HypergraphStore::new();
        let id = seeded_carrier_node(&mut graph);
        // Flip decomposition-fire-tick's own latch and write its real value
        // — exactly what `decomposition.bsl:309-310` does the SAME tick
        // (`countdown.rs`'s own module doc: "both written in the SAME
        // effects block"). control-crisis-tick stays seeded 0 / not-yet-
        // latched — its own beat (CONTROL_RATIO_CRISIS) has not fired yet.
        graph
            .update_node(id, "institution/decomposition-fire-tick", 53.0)
            .expect("decomposition-fire-tick fires at 53");
        graph
            .update_node(id, "institution/decomposition-fired-known", 1.0)
            .expect("decomposition-fired-known flips to 1");
        let roster = vec![("carceral-register".to_owned(), id, NodeKind::Institution)];
        let rendered = format_roster_panel(&graph, &roster, Some(0));
        assert!(
            rendered.contains("institution/decomposition-fire-tick: 53.00"),
            "once the latch flips the real material value must render, got {rendered:?}"
        );
        assert!(
            rendered.contains("institution/control-crisis-tick: not computed by this port"),
            "control-crisis-tick's own latch has not flipped — still honest not-yet-latched, \
             got {rendered:?}"
        );
    }

    #[test]
    fn selecting_wraps_at_both_ends() {
        let mut selected = SelectedRosterIndex(None);
        let len = 3;
        // Down from None picks index 0, then wraps 0 -> 1 -> 2 -> 0.
        selected.0 = Some(selected.0.map_or(0, |i| (i + 1) % len));
        assert_eq!(selected.0, Some(0));
        for _ in 0..len {
            selected.0 = Some(selected.0.map_or(0, |i| (i + 1) % len));
        }
        assert_eq!(selected.0, Some(0));

        // Up from None ALSO picks index 0 (never -1) — the first press must
        // land on a real roster entry regardless of direction.
        let mut selected = SelectedRosterIndex(None);
        selected.0 = Some(selected.0.map_or(0, |i| (i + len - 1) % len));
        assert_eq!(selected.0, Some(0));
    }
}
