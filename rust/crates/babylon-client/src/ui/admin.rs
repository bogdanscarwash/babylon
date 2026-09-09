//! The declared admin surface (B3 wave-1 Task 3, plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md`
//! §2.6/§3.3): a persistent banner naming this build's projection as
//! unfogged material truth — the NAMED exception Global Constraint 5
//! licenses (no player exists yet, so there is no epistemic state to
//! protect) — plus an `F3`-toggled panel rendering two instruments that
//! cost nothing new to show: the engine's own `TickReport` (computed every
//! tick by `EngineSession::advance` and discarded until this file binds
//! it — `ui::time::advance_ticks` is the sole writer of [`LastTickReport`])
//! and a raw attribute dump of the selected node, read straight off the
//! graph. The dump deliberately bypasses `crate::projection::Projector`:
//! the whole point of a raw admin dump is showing what the graph actually
//! holds, unfiltered by any provenance classification.

use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::engine_link::EngineSession;
use babylon_graph::state_hash::CanonicalState;
use bevy::prelude::*;

/// The banner's own literal text (§2.6) — this build renders material
/// truth unfogged, and says so on screen rather than leaving an unfogged
/// panel unlabeled.
pub const BANNER_TEXT: &str = "ADMIN \u{b7} MATERIAL TRUTH \u{b7} UNFOGGED";

#[derive(Component)]
pub struct AdminBanner;

/// `Startup` system: spawns the persistent admin banner. Always visible —
/// unlike the `F3` panel below, this is not optional detail; it is the
/// disclosure the whole wave-1 projection depends on (§2.6: "the admin
/// surface is the NAMED exception").
pub fn spawn_admin_banner(mut commands: Commands) {
    commands.spawn((
        Text::new(BANNER_TEXT),
        TextColor(crate::palette::CRIMSON),
        Node {
            position_type: PositionType::Absolute,
            top: px(4),
            left: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::AdminDisclosure),
        AdminBanner,
    ));
}

/// The most recent tick's `TickReport` — computed every tick by
/// `EngineSession::advance` and, before this file, thrown away at the call
/// site (`ui::time::advance_ticks` never bound the `Ok` value). This
/// resource is that binding: `advance_ticks` is its sole writer, once per
/// tick inside its own batch loop, so after a multi-tick catch-up batch
/// this holds exactly the LAST tick's report — the report for the tick the
/// rest of the HUD is also showing. `None` before the first advance.
#[derive(Resource, Default)]
pub struct LastTickReport(pub Option<babylon_tick::TickReport>);

/// Whether the `F3` admin panel (`TickReport` + roster dump) is showing.
/// Off by default: an unrequested admin view is not the first thing a
/// fresh run should show, matching this crate's own "recallable, not
/// forced open" precedent (`ui::time`'s controls readout, plan §2.5).
#[derive(Resource, Default)]
pub struct AdminPanelVisible(pub bool);

#[derive(Component)]
pub struct AdminPanelText;

/// `Startup` system: spawns the (initially empty) `F3` panel text entity.
pub fn spawn_admin_panel(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextColor(crate::palette::BONE),
        Node {
            position_type: PositionType::Absolute,
            top: px(24),
            left: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::AdminInspector),
        AdminPanelText,
    ));
}

/// `Update` system: `F3` flips [`AdminPanelVisible`].
pub fn toggle_admin_panel(keys: Res<ButtonInput<KeyCode>>, mut visible: ResMut<AdminPanelVisible>) {
    if keys.just_pressed(KeyCode::F3) {
        visible.0 = !visible.0;
    }
}

/// Renders the per-rule breakdown from a `TickReport` — pure and
/// independently testable. Governed phase placement is the engine's contract
/// on `per_rule_fired`; D16 bytes order same-position ties. This function
/// renders the vector as given and never re-sorts it, so the display inherits
/// engine order rather than duplicating the scheduler. Every number here is a
/// field `TickReport` already carries.
#[must_use]
pub fn format_tick_report(report: &babylon_tick::TickReport) -> String {
    let mut lines = vec![format!("tick report \u{2014} {} fired", report.fired)];
    for (rule_id, fired) in &report.per_rule_fired {
        lines.push(format!("  {rule_id}: {fired}"));
    }
    lines.join("\n")
}

/// Renders a raw attribute dump for one node, sorted by field name for a
/// stable, readable, testable render (`CanonicalState::all_attributes`
/// makes no ordering promise of its own — its own doc says "in any
/// order"). This is the one place in the crate that reads a node's WHOLE
/// attribute set rather than one named field at a time — an admin-only
/// instrument, deliberately not routed through `Projector`.
#[must_use]
pub fn format_roster_dump(label: &str, mut rows: Vec<(String, f64)>) -> String {
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut lines = vec![format!("roster \u{2014} {label}")];
    for (field, value) in rows {
        lines.push(format!("  {field}: {value}"));
    }
    lines.join("\n")
}

/// `Update` system: repaints [`AdminPanelText`] from [`AdminPanelVisible`],
/// [`LastTickReport`] and the selected demo county's own raw attribute
/// set. Renders nothing (`""`) while hidden — the same "empty string is
/// the honest render of nothing to show" idiom
/// `loop_ui::refresh_state_panel` already established for this crate, and
/// the early return means the (tiny but nonzero) attribute-dump work never
/// runs on a frame nobody can see it.
pub fn refresh_admin_panel(
    visible: Res<AdminPanelVisible>,
    last_report: Res<LastTickReport>,
    selected: Res<crate::map::SelectedCounty>,
    session: Res<EngineSession>,
    atlas: Res<crate::atlas::CountyAtlas>,
    mut panel_text: Query<&mut Text, With<AdminPanelText>>,
) {
    let Ok(mut text) = panel_text.single_mut() else {
        return;
    };
    if !visible.0 {
        text.0 = String::new();
        return;
    }

    let mut sections = Vec::new();
    match &last_report.0 {
        Some(report) => sections.push(format_tick_report(report)),
        None => sections.push("tick report \u{2014} not yet run".to_owned()),
    }

    match crate::loop_ui::selected_demo_node(&atlas, &selected, &session.roster) {
        Some((fips, name, id)) => {
            let rows: Vec<(String, f64)> = session
                .inner
                .graph()
                .all_attributes()
                .into_iter()
                .filter(|(node_id, _, _)| *node_id == id)
                .map(|(_, field, value)| (field, value))
                .collect();
            sections.push(format_roster_dump(&format!("{name} ({fips})"), rows));
        }
        None => sections.push("roster \u{2014} no county selected".to_owned()),
    }

    text.0 = sections.join("\n\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tick_report_renders_the_total_and_every_rule_in_the_given_order() {
        let report = babylon_tick::TickReport {
            before: [0u8; 32],
            after: [1u8; 32],
            world_before: [2u8; 32],
            world_after: [3u8; 32],
            considered: 24,
            fired: 18,
            per_rule_considered: vec![
                ("lifecycle/dpd-circuit".to_owned(), 16),
                ("vitality/subsistence-and-death".to_owned(), 8),
            ],
            per_rule_fired: vec![
                ("lifecycle/dpd-circuit".to_owned(), 12),
                ("vitality/subsistence-and-death".to_owned(), 6),
            ],
            audit_receipts: Vec::new(),
            choice_receipts: Vec::new(),
            committed_events: Vec::new(),
        };
        let rendered = format_tick_report(&report);
        assert_eq!(
            rendered,
            "tick report \u{2014} 18 fired\n  lifecycle/dpd-circuit: 12\n  \
             vitality/subsistence-and-death: 6"
        );
    }

    #[test]
    fn format_roster_dump_sorts_by_field_name() {
        let rows = vec![
            ("territory/pop-p".to_owned(), 5748.0),
            ("territory/pop-d".to_owned(), 2042.0),
        ];
        let rendered = format_roster_dump("01001", rows);
        assert_eq!(
            rendered,
            "roster \u{2014} 01001\n  territory/pop-d: 2042\n  territory/pop-p: 5748"
        );
    }
}
