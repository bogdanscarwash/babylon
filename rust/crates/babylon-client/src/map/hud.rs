//! The county HUD (Task 11, completing B1's never-built Task 10, extended
//! past B1's own spec with the active-lens label this plan's own honesty
//! rule adds — Task 10's finding that CRIMSON carries three separate
//! meanings across the three lenses means the map must always say which
//! lens is live).
//!
//! **Forward-reference note.** `TickCounter` (Task 14, `loop_ui.rs`) does
//! not exist yet at this point in the plan's own task order — this module
//! owns its OWN minimal `HudTick` resource instead of reaching for a type
//! that hasn't landed, so this task's tests do not need to wait on Phase D.
//! Task 14 keeps `HudTick` in sync alongside `TickCounter` when it lands.

use crate::atlas::CountyAtlas;
use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::lens::CurrentLensData;
use crate::map::bands::{ActiveLens, LENSES};
use crate::map::pick::{HoveredCounty, SelectedCounty};
use crate::palette;
use bevy::prelude::*;

// FB5: production code reads the atlas through `Res<crate::atlas::CountyAtlas>`
// now — this file's own tests still build fixtures directly from the
// embedded bytes, so the const stays, scoped to `cfg(test)`.
#[cfg(test)]
const ATLAS_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../assets/map/county_atlas.bin"
));

/// The tick number the lens readouts should quote as "live, tick N" —
/// owned here rather than borrowed from `loop_ui::TickCounter` (see the
/// module doc's forward-reference note). Defaults to `0` (tick 0, the
/// scenario's own seeded, un-ticked state).
#[derive(Resource, Default)]
pub struct HudTick(pub i64);

/// The name a Tab press cycles through, in cycle order — derived from
/// `LENSES` (B3 wave-1 Task 8, §2.10) so the footer text and the picker
/// (Task 12) can never drift apart from the registry itself.
#[must_use]
pub fn lens_label(lens: ActiveLens) -> &'static str {
    LENSES[lens.0].label
}

/// One line naming the active lens's own reading for a specific county —
/// the three lenses phrase their own value differently (Tension: signed
/// `w` with its pole named in words; Legitimation: the classification word
/// plus the live tick; Population Trend: the signed delta plus
/// growing/declining plus the live tick), and "no data this tick" when the
/// cell is absent, regardless of which lens is active.
///
/// **The one thing the registry cannot make generic (B3 wave-1 Task 8,
/// §2.10).** `LensSpec::label`/`::help`/`::paint` collapse into table
/// lookups; this function's per-lens PHRASING does not — dispatches on
/// `LENSES[active.0].id` (a stable string, not the old closed enum) rather
/// than a magic index, and a new registry row still needs a new arm here,
/// exactly as `LensSpec::id`'s own doc says.
///
/// # Panics
/// If `LENSES[active.0].id` names a lens with a real reading (`Some`) but
/// no arm below knows how to phrase it — loud by design (III.11): a silent
/// fallback would print nonsense for a lens nobody taught this function
/// to describe.
#[must_use]
pub fn format_lens_line(active: ActiveLens, cell: Option<f64>, tick: i64) -> String {
    let spec = &LENSES[active.0];
    let label = lens_label(active);
    match (spec.id, cell) {
        ("county_tension", Some(w)) => {
            let side = if w < 0.0 {
                "\u{3a6}-source, bled"
            } else if w > 0.0 {
                "\u{3a6}-recipient, bribed"
            } else {
                "neither pole"
            };
            format!("Lens: {label} \u{2014} w = {w:.2} ({side})")
        }
        ("county_legitimation", Some(class)) => {
            let word = crate::lens::classify(class);
            let word = match word {
                crate::lens::LegitimationClass::Stable => "STABLE",
                crate::lens::LegitimationClass::Unstable => "UNSTABLE",
                crate::lens::LegitimationClass::Crisis => "CRISIS",
            };
            format!("Lens: {label} \u{2014} {word} (live, tick {tick})")
        }
        ("county_population_trend", Some(delta)) => {
            let direction = if delta > 0.0 {
                "growing"
            } else if delta < 0.0 {
                "declining"
            } else {
                "unchanged"
            };
            format!(
                "Lens: {label} \u{2014} {delta:+.0} since tick 0 ({direction}, live, tick {tick})"
            )
        }
        (_, None) => format!("Lens: {label} \u{2014} no data this tick"),
        (id, Some(_)) => panic!(
            "format_lens_line: lens id {id:?} has a real reading but no known text-format arm \
             — a new LENSES row needs a new arm here too (§2.10's own acknowledged exception: \
             not everything collapses into the table)"
        ),
    }
}

/// The persistent DIM footer naming the cycle order — derived from
/// `LENSES` (B3 wave-1 Task 8, §2.10) rather than hand-maintained, so a new
/// registry row can never leave this string stale the way a hand-written
/// `const` could. Task 12 wires the `Tab` key to this exact order, and this
/// string is the one place a player is told it.
#[must_use]
pub fn lens_cycle_footer() -> String {
    let mut footer = "Tab: ".to_owned();
    for (i, spec) in LENSES.iter().enumerate() {
        if i > 0 {
            footer.push_str(" \u{2192} ");
        }
        footer.push_str(spec.label);
    }
    footer.push_str(" \u{2192} ");
    footer.push_str(LENSES[0].label);
    footer
}

fn lens_cell(reading: &crate::lens::LensReading, fips: &str) -> Option<f64> {
    reading
        .cells
        .iter()
        .find(|(cell_fips, _)| cell_fips == fips)
        .and_then(|(_, v)| *v)
}

/// Which `(fips, county name)` the HUD should describe this frame: the
/// hovered county if the cursor is over one, else the last-selected one,
/// else nothing.
fn active_county(
    atlas: &CountyAtlas,
    hovered: &HoveredCounty,
    selected: &SelectedCounty,
) -> Option<(String, String)> {
    let idx = hovered.0.or(selected.0)?;
    let county = atlas.county(idx)?;
    Some((county.fips.to_owned(), county.name.to_owned()))
}

#[derive(Component)]
pub struct CountyHudText;

#[derive(Component)]
pub struct AbsenceBanner;

#[derive(Component)]
pub struct LensFooter;

/// `Startup` system: spawns the three HUD text entities.
pub(super) fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextColor(palette::BONE),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(24),
            left: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::CountyLensHud),
        CountyHudText,
    ));
    commands.spawn((
        Text::new(""),
        TextColor(palette::CRIMSON),
        Node {
            position_type: PositionType::Absolute,
            top: px(24),
            left: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::CountyLensHud),
        AbsenceBanner,
    ));
    commands.spawn((
        Text::new(lens_cycle_footer()),
        TextColor(palette::DIM),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(4),
            left: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::CountyLensHud),
        LensFooter,
    ));
}

/// `Update` system: repaints the county text and the absence banner from
/// `HoveredCounty`/`SelectedCounty`, `ActiveLens` and `CurrentLensData`.
/// Reads the atlas through `Res<CountyAtlas>` (FB5 fix — this system ran
/// EVERY Update frame unconditionally and used to re-parse the embedded
/// 1.7 MB atlas — a full SHA-256 hash plus a table decode — on every one
/// of them; `map::mesh::spawn_map_surface` parses it exactly once, at
/// Startup, and this system now shares that parse).
// `Res<CountyAtlas>` (FB5) pushed this over clippy's 7-argument default —
// every parameter is a distinct, narrow Bevy `SystemParam` the scheduler
// inspects individually for parallel-access analysis, not a bundle of
// unrelated state a real function signature would want collapsed; a
// `#[derive(SystemParam)]` wrapper struct would hide the same seven
// `Res`/`Query` handles behind one name without changing what the system
// actually reads, so it is not worth the indirection here.
#[allow(clippy::too_many_arguments)]
pub(crate) fn refresh_hud(
    hovered: Res<HoveredCounty>,
    selected: Res<SelectedCounty>,
    active: Res<ActiveLens>,
    lens_data: Res<CurrentLensData>,
    tick: Res<HudTick>,
    atlas: Res<CountyAtlas>,
    mut county_text: Query<&mut Text, (With<CountyHudText>, Without<AbsenceBanner>)>,
    mut banner_text: Query<&mut Text, With<AbsenceBanner>>,
) {
    let reading = &lens_data.0[active.0];

    if let Ok(mut text) = county_text.single_mut() {
        text.0 = match active_county(&atlas, &hovered, &selected) {
            Some((fips, name)) => {
                let cell = lens_cell(reading, &fips);
                format!(
                    "{name} ({fips})\n{}",
                    format_lens_line(*active, cell, tick.0)
                )
            }
            None => String::new(),
        };
    }

    if let Ok(mut banner) = banner_text.single_mut() {
        banner.0 = reading.absent_reason.clone().unwrap_or_default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lens::LensReading;

    #[test]
    fn tension_line_names_the_pole_in_words() {
        assert_eq!(
            format_lens_line(ActiveLens(0), Some(-0.42), 7),
            "Lens: Tension \u{2014} w = -0.42 (\u{3a6}-source, bled)"
        );
        assert_eq!(
            format_lens_line(ActiveLens(0), Some(0.42), 7),
            "Lens: Tension \u{2014} w = 0.42 (\u{3a6}-recipient, bribed)"
        );
    }

    #[test]
    fn legitimation_line_names_the_class_word_and_the_tick() {
        assert_eq!(
            format_lens_line(ActiveLens(1), Some(2.0), 7),
            "Lens: Legitimation \u{2014} CRISIS (live, tick 7)"
        );
        assert_eq!(
            format_lens_line(ActiveLens(1), Some(0.0), 7),
            "Lens: Legitimation \u{2014} STABLE (live, tick 7)"
        );
    }

    #[test]
    fn population_trend_line_names_the_direction_and_the_tick() {
        assert_eq!(
            format_lens_line(ActiveLens(2), Some(37.0), 5),
            "Lens: Population Trend \u{2014} +37 since tick 0 (growing, live, tick 5)"
        );
        assert_eq!(
            format_lens_line(ActiveLens(2), Some(-19.0), 5),
            "Lens: Population Trend \u{2014} -19 since tick 0 (declining, live, tick 5)"
        );
    }

    #[test]
    fn any_lens_with_no_cell_reports_no_data_this_tick() {
        assert_eq!(
            format_lens_line(ActiveLens(0), None, 3),
            "Lens: Tension \u{2014} no data this tick"
        );
        assert_eq!(
            format_lens_line(ActiveLens(2), None, 3),
            "Lens: Population Trend \u{2014} no data this tick"
        );
    }

    #[test]
    fn active_county_prefers_hovered_over_selected() {
        let atlas = CountyAtlas::parse(ATLAS_BYTES).expect("committed atlas parses");
        let hovered = HoveredCounty(Some(0));
        let selected = SelectedCounty(Some(1));
        let (fips, _name) = active_county(&atlas, &hovered, &selected).expect("some county");
        assert_eq!(fips, atlas.county(0).unwrap().fips);
    }

    #[test]
    fn active_county_falls_back_to_selected_when_nothing_is_hovered() {
        let atlas = CountyAtlas::parse(ATLAS_BYTES).expect("committed atlas parses");
        let hovered = HoveredCounty(None);
        let selected = SelectedCounty(Some(1));
        let (fips, _name) = active_county(&atlas, &hovered, &selected).expect("some county");
        assert_eq!(fips, atlas.county(1).unwrap().fips);
    }

    #[test]
    fn active_county_is_none_when_nothing_is_hovered_or_selected() {
        let atlas = CountyAtlas::parse(ATLAS_BYTES).expect("committed atlas parses");
        assert!(active_county(&atlas, &HoveredCounty(None), &SelectedCounty(None)).is_none());
    }

    #[test]
    fn lens_cell_looks_up_by_fips() {
        let reading = LensReading {
            cells: vec![("00001".to_owned(), Some(1.5)), ("00002".to_owned(), None)],
            absent_reason: None,
        };
        assert_eq!(lens_cell(&reading, "00001"), Some(1.5));
        assert_eq!(lens_cell(&reading, "00002"), None);
        assert_eq!(lens_cell(&reading, "99999"), None);
    }
}
