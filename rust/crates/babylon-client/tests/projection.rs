//! B3 wave-1 Task 3 (plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md` §2.6,
//! RED phase): the projection seam's own contract, exercised as a
//! consumer of `babylon_client`'s public API — `Projector::material().read`
//! for the three provenances this crate can produce today
//! (`Material`/`Absent`/`NotComputed`), the digit-free render discipline
//! (III.11), the `Redacted` variant's declared-dead sentinel (I9), and the
//! admin banner's own rendered text.

use babylon_client::projection::{Projector, Provenance};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

#[test]
fn material_read_returns_material_provenance_and_the_written_value() {
    let mut graph = HypergraphStore::new();
    let id = graph.add_node("TERRITORY").expect("add territory");
    graph
        .update_node(id, "territory/pop-d", 2042.0)
        .expect("stamp pop-d");

    let reading = Projector::material().read(&graph, id, "territory/pop-d");
    assert_eq!(reading.value, Some(2042.0));
    assert_eq!(reading.provenance, Provenance::Material);
}

#[test]
fn an_unwritten_field_returns_absent_never_a_fabricated_zero() {
    let mut graph = HypergraphStore::new();
    let id = graph.add_node("TERRITORY").expect("add territory");
    // Nothing ever stamps territory/pop-p on this node.

    let reading = Projector::material().read(&graph, id, "territory/pop-p");
    assert_eq!(
        reading.value, None,
        "an unwritten field must read back as None — Some(0.0) would be exactly \
         the fabrication III.11 forbids"
    );
    assert!(matches!(reading.provenance, Provenance::Absent { .. }));
}

/// §2.6's I2 table: a field a port DECLARED it will never compute renders
/// its reason and contains no digit — never the numeral it would
/// otherwise read as (`decomposition.bsl:264`'s bare `0.0c`, transcribed
/// as the `SUPERWAGE_CRISIS.desired-wages` key `Projector::material`
/// declares unconditionally, checked before any graph read).
#[test]
fn a_declared_not_computed_key_renders_its_reason_with_no_digit() {
    let graph = HypergraphStore::new();
    let reading = Projector::material().read(&graph, NodeId(0), "SUPERWAGE_CRISIS.desired-wages");
    assert_eq!(reading.value, None);
    assert!(matches!(reading.provenance, Provenance::NotComputed { .. }));

    let rendered = reading.render(0);
    assert!(
        !rendered.chars().any(|c| c.is_ascii_digit()),
        "a NotComputed render must contain no digit, got {rendered:?}"
    );
    assert!(rendered.contains("not computed by this port"));
}

/// I9: `Provenance::Redacted` is declared-dead until #593.
/// `src/projection.rs` itself is exempt (it declares the variant and its
/// own exhaustive `render` match needs to name it) — the SAME
/// whole-file-exemption shape
/// `tests/unit/render/test_rust_theme_parity.py` already uses for
/// `palette.rs`'s own `Color::srgb_u8` literals. Every OTHER file under
/// `src/` must never construct this variant; the day a fixture does
/// (exactly revision 1's own corrected mistake, plan §2.6 I9), this test
/// goes red.
#[test]
fn redacted_is_declared_dead_until_593() {
    let src_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let exempt = src_root.join("projection.rs");
    let mut offenders = Vec::new();
    scan_dir(&src_root, &exempt, &mut offenders, 0);
    assert!(
        offenders.is_empty(),
        "Provenance::Redacted must not be constructed outside src/projection.rs \
         (I9 — declared-dead until #593), found: {offenders:?}"
    );
}

/// A bounded recursive walk (Power-of-10 rule 2): `MAX_SCAN_DEPTH` is a
/// generous ceiling on this crate's own `src/` module nesting, asserted
/// loudly rather than silently exceeded — never an unbounded traversal.
const MAX_SCAN_DEPTH: usize = 8;

fn scan_dir(
    dir: &std::path::Path,
    exempt: &std::path::Path,
    offenders: &mut Vec<String>,
    depth: usize,
) {
    assert!(
        depth <= MAX_SCAN_DEPTH,
        "src/ tree deeper than MAX_SCAN_DEPTH ({MAX_SCAN_DEPTH}) at {} — raise the \
         constant deliberately, this is not meant to loop unbounded",
        dir.display()
    );
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, exempt, offenders, depth + 1);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") && path != exempt {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            if text.contains("Provenance::Redacted") {
                offenders.push(path.display().to_string());
            }
        }
    }
}

/// A headless assertion that the admin banner entity exists and reads the
/// declared text — `ADMIN · MATERIAL TRUTH · UNFOGGED` (§2.6/§3.3): the
/// NAMED exception this wave's whole unfogged projection depends on
/// (Global Constraint 5).
#[test]
fn the_admin_banner_entity_exists_and_reads_the_declared_text() {
    let mut app = new_app();
    app.update(); // Startup

    let world = app.world_mut();
    let mut query = world.query_filtered::<&Text, With<babylon_client::ui::admin::AdminBanner>>();
    let text = query
        .single(world)
        .expect("exactly one admin banner entity")
        .0
        .clone();
    assert_eq!(text, "ADMIN \u{b7} MATERIAL TRUTH \u{b7} UNFOGGED");
    assert_eq!(text, babylon_client::ui::admin::BANNER_TEXT);
}

// ---- Review fix round 1 (task-3-review.md, Important-1) ----
//
// `toggle_admin_panel`/`refresh_admin_panel` had zero headless real-`App`/
// real-`KeyboardInput` coverage — the plan's own §2.8 binding standard
// ("every new UI system in this train gets a test at this layer, or it
// does not land") was unmet for these two systems. The two tests below
// close that gap, following the house pattern `tests/time_controls.rs`
// set for Task 2's own wiring.

/// The real app: `MapPlugin` + `TickLoopPlugin` together, exactly as
/// `main.rs` wires them and every other test file in this crate builds
/// them.
fn new_app() -> App {
    use bevy::asset::AssetPlugin;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    // B3 wave-1 Task 5 (plan §2.5 Minor 7): `SelectedStory` has no
    // `Default` — every app-builder must say which story it wants.
    app.insert_resource(babylon_client::story::SelectedStory(
        babylon_client::story::counties(),
    ));
    app
}

/// Presses `key` through the REAL `KeyboardInput` message pipeline —
/// necessary, not stylistic, once `MapPlugin` is in the App (every other
/// test file in this crate's own module docs has the full citation: a
/// direct `ButtonInput::press()` call from test code is wiped by
/// `InputPlugin`'s `PreUpdate` clear before an `Update` system ever
/// observes it).
fn press_key_via_real_event(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<Messages<KeyboardInput>>()
        .write(KeyboardInput {
            key_code: key,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
}

fn release_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
}

fn admin_panel_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query =
        world.query_filtered::<&Text, With<babylon_client::ui::admin::AdminPanelText>>();
    query
        .single(world)
        .expect("exactly one admin panel text entity")
        .0
        .clone()
}

/// `F3` must flip `AdminPanelVisible` AND that flip must be visible in the
/// REAL rendered `AdminPanelText` — empty while hidden (the same "empty
/// string is the honest render of nothing to show" idiom
/// `loop_ui::refresh_state_panel` already established), non-empty once
/// shown, empty again on a second press. `TimeUpdateStrategy::
/// ManualDuration(Duration::ZERO)` is pinned before the FIRST `app.update()`
/// because `RunState.running` defaults `true` (I4 — an unpinned wall-clock
/// delta could cross a tick boundary and advance the engine, which this
/// test never asks for; `time_controls.rs` row 4's own comment names the
/// same hazard).
#[test]
fn f3_toggles_the_admin_panel_visible_and_hidden_across_real_update_cycles() {
    let mut app = new_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.update(); // Startup

    assert!(
        !app.world()
            .resource::<babylon_client::ui::admin::AdminPanelVisible>()
            .0,
        "the admin panel must start hidden"
    );
    assert_eq!(
        admin_panel_text(&mut app),
        "",
        "a hidden panel must render nothing"
    );

    press_key_via_real_event(&mut app, KeyCode::F3);
    app.update();
    release_key(&mut app, KeyCode::F3);

    assert!(
        app.world()
            .resource::<babylon_client::ui::admin::AdminPanelVisible>()
            .0,
        "F3 must reveal the panel"
    );
    let shown = admin_panel_text(&mut app);
    assert!(
        !shown.is_empty(),
        "a shown panel must render something, got empty text"
    );

    press_key_via_real_event(&mut app, KeyCode::F3);
    app.update();
    release_key(&mut app, KeyCode::F3);

    assert!(
        !app.world()
            .resource::<babylon_client::ui::admin::AdminPanelVisible>()
            .0,
        "a second F3 press must hide the panel again"
    );
    assert_eq!(
        admin_panel_text(&mut app),
        "",
        "a re-hidden panel must render nothing again"
    );
}

/// The per-rule breakdown must render through the REAL `refresh_admin_panel`
/// system reading a REAL `LastTickReport` resource — not just
/// `format_tick_report` in isolation (already covered by `ui::admin::tests`
/// in `src/ui/admin.rs`). Seeds the resource directly rather than driving
/// real ticks: deterministic, and it isolates the WIRING under test from
/// the engine's own numbers.
#[test]
fn the_admin_panel_renders_the_per_rule_breakdown_from_a_seeded_tick_report() {
    let mut app = new_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.update(); // Startup

    app.world_mut()
        .resource_mut::<babylon_client::ui::admin::LastTickReport>()
        .0 = Some(babylon_tick::TickReport {
        before: [0u8; 32],
        after: [1u8; 32],
        world_before: [2u8; 32],
        world_after: [3u8; 32],
        considered: 11,
        fired: 7,
        per_rule_considered: vec![
            ("lifecycle/dpd-circuit".to_owned(), 7),
            ("vitality/subsistence-and-death".to_owned(), 4),
        ],
        per_rule_fired: vec![
            ("lifecycle/dpd-circuit".to_owned(), 5),
            ("vitality/subsistence-and-death".to_owned(), 2),
        ],
        audit_receipts: Vec::new(),
        choice_receipts: Vec::new(),
        committed_events: Vec::new(),
    });

    press_key_via_real_event(&mut app, KeyCode::F3);
    app.update();
    release_key(&mut app, KeyCode::F3);

    let text = admin_panel_text(&mut app);
    assert!(
        text.contains("tick report \u{2014} 7 fired"),
        "got {text:?}"
    );
    assert!(
        text.contains("lifecycle/dpd-circuit: 5"),
        "the per-rule breakdown must render through the REAL system, got {text:?}"
    );
    assert!(
        text.contains("vitality/subsistence-and-death: 2"),
        "got {text:?}"
    );
}
