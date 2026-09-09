//! B3 wave-1 Task 8's own RED phase (plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md` §2.10):
//! the lens registry — `LENSES: &'static [LensSpec]` replacing the closed
//! `ActiveLens` enum plus five files' worth of exhaustive matches
//! (`map/bands.rs`, `map/mod.rs`, `map/hud.rs`). Five assertions, mirroring
//! the task brief's own RED list verbatim:
//!
//! 1. `LENSES` ids are unique and non-empty.
//! 2. Every spec's `help` names at least one engine field string that
//!    genuinely appears in `lens.rs` (cross-checked against the `pub`
//!    field-name consts declared there, not a second hand-copied literal).
//! 3. `Tab` visits every index exactly once per full cycle then returns to
//!    the start — generalized to `LENSES.len()`, not hardcoded to three,
//!    so this test keeps proving the claim if the registry ever grows.
//! 4. The derived footer string (`lens_cycle_footer()`) contains every
//!    lens label.
//! 5. `CurrentLensData.len() == LENSES.len()`.
//!
//! RED→GREEN record: authored RED when `LensSpec`/`LensPaint`/`LensInputs`,
//! `LENSES`, `lens_cycle_footer` and `ActiveLens(usize)` did not exist
//! (every reference below failed to resolve, the train's usual "module
//! absent" RED-commit shape); GREEN since the registry landed — this file
//! is the registry's live regression guard, not a placeholder.

use babylon_client::lens::{
    LEGITIMATION_CRISIS_FIELD, POP_D_FIELD, POP_D_PRIME_FIELD, POP_P_FIELD, TENSION_E_FIELD,
    TENSION_S_FIELD,
};
use babylon_client::map::{ActiveLens, MapPlugin, LENSES};
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::prelude::*;

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

// ---- 8.1(1): ids are unique and non-empty ----

#[test]
fn lens_ids_are_unique_and_non_empty() {
    let mut seen = std::collections::HashSet::new();
    for spec in LENSES {
        assert!(!spec.id.is_empty(), "a lens id must not be empty");
        assert!(
            seen.insert(spec.id),
            "duplicate lens id {:?} — every row in LENSES must be independently addressable",
            spec.id
        );
    }
}

// ---- 8.1(2): every help names a real engine field ----

#[test]
fn every_spec_help_names_a_real_engine_field_from_lens_rs() {
    // The full set of `pub` field-name consts `crate::lens` declares — the
    // SAME strings `county_tension`/`county_legitimation`/
    // `county_population_trend` actually read off the graph. Cross-checking
    // against these (not a second hand-typed literal) is what makes this a
    // genuine regression guard: an edit that renames a field constant
    // without updating the matching `help` string goes red here.
    let known_fields = [
        TENSION_E_FIELD,
        TENSION_S_FIELD,
        LEGITIMATION_CRISIS_FIELD,
        POP_D_FIELD,
        POP_P_FIELD,
        POP_D_PRIME_FIELD,
    ];
    for spec in LENSES {
        assert!(
            known_fields.iter().any(|field| spec.help.contains(field)),
            "lens {:?}'s help {:?} names no known engine field from lens.rs — honest-physics \
             discipline (plan §1) requires every lens's help to name a real field",
            spec.id,
            spec.help
        );
    }
}

// ---- 8.1(3): Tab visits every index exactly once, then returns ----

#[test]
fn tab_visits_every_lens_index_exactly_once_then_returns_to_start() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(MapPlugin);
    app.update(); // Startup.

    let start = app.world().resource::<ActiveLens>().0;
    let mut seen = vec![start];
    for _ in 0..LENSES.len() {
        press_key_via_real_event(&mut app, KeyCode::Tab);
        app.update();
        // Same not-yet-released-between-taps gotcha `map/mod.rs`'s own
        // test module doc names: release directly rather than through a
        // second event + update cycle.
        release_key(&mut app, KeyCode::Tab);
        seen.push(app.world().resource::<ActiveLens>().0);
    }

    // `seen` has LENSES.len() + 1 entries: the start plus one per press.
    // The first LENSES.len() entries (start included) must be a
    // permutation of every valid index, and pressing Tab one more time
    // than there are lenses must land back on the start.
    let mut visited: Vec<usize> = seen[..LENSES.len()].to_vec();
    visited.sort_unstable();
    let expected: Vec<usize> = (0..LENSES.len()).collect();
    assert_eq!(
        visited, expected,
        "a full Tab cycle must visit every registered lens index exactly once"
    );
    assert_eq!(
        *seen.last().expect("at least one press happened"),
        start,
        "LENSES.len() presses from the start must return to the start"
    );
}

// ---- 8.1(4): the derived footer names every label ----

#[test]
fn the_derived_footer_names_every_lens_label() {
    let footer = babylon_client::map::lens_cycle_footer();
    for spec in LENSES {
        assert!(
            footer.contains(spec.label),
            "footer {footer:?} is missing label {:?}",
            spec.label
        );
    }
}

// ---- 8.1(5): CurrentLensData carries one reading per registered lens ----

#[test]
fn current_lens_data_has_one_reading_per_registered_lens() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    // B3 wave-1 Task 5 (plan §2.5 Minor 7): `SelectedStory` has no
    // `Default` — every app-builder must say which story it wants.
    app.insert_resource(babylon_client::story::SelectedStory(
        babylon_client::story::counties(),
    ));
    app.update(); // Startup — spawn_engine_session_and_hud inserts CurrentLensData.

    let lens_data = app
        .world()
        .resource::<babylon_client::lens::CurrentLensData>();
    assert_eq!(
        lens_data.0.len(),
        LENSES.len(),
        "CurrentLensData must carry exactly one LensReading per registered lens"
    );
}
