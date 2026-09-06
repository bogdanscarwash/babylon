//! Bounded measurements of real native frame intervals. This observes client
//! scheduling and viewport state only; it never reads simulation state.

use babylon_persistence::CampaignId;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::observer::{ObserverSession, Perspective, SessionPhase};
use crate::observer_ui::{ObserverUiState, ObserverViewport};
use crate::production::{PrimaryView, ProductionNavigation};

const SAMPLE_COUNT: usize = 300;
const SETTLE_SECONDS: f64 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderMode {
    Map3d,
    Production3d,
    Production2d,
}

impl RenderMode {
    fn label(self) -> &'static str {
        match self {
            Self::Map3d => "map3d",
            Self::Production3d => "production3d",
            Self::Production2d => "production2d",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
// Independent presentation controls define the scope of comparable samples.
#[allow(clippy::struct_excessive_bools)]
struct SamplingScope {
    mode: RenderMode,
    campaign: CampaignId,
    perspective: Perspective,
    window_pixels: UVec2,
    viewport_origin: UVec2,
    viewport_pixels: UVec2,
    ui_scale_bits: u32,
    history_open: bool,
    archive_open: bool,
    details_open: bool,
    reduced_motion: bool,
    playing: bool,
}

#[derive(Debug)]
struct FrameReport {
    median_ms: f64,
    p95_ms: f64,
    fps: f64,
    elapsed_seconds: f64,
    phases: [u32; 7],
}

#[derive(Resource)]
struct FrameSampler {
    scope: Option<SamplingScope>,
    settle_remaining: f64,
    intervals: [f64; SAMPLE_COUNT],
    count: usize,
    phases: [u32; 7],
}

impl Default for FrameSampler {
    fn default() -> Self {
        Self {
            scope: None,
            settle_remaining: SETTLE_SECONDS,
            intervals: [0.0; SAMPLE_COUNT],
            count: 0,
            phases: [0; 7],
        }
    }
}

fn phase_index(phase: SessionPhase) -> usize {
    match phase {
        SessionPhase::Connecting => 0,
        SessionPhase::Loading => 1,
        SessionPhase::Ready => 2,
        SessionPhase::Advancing => 3,
        SessionPhase::Complete => 4,
        SessionPhase::Failed => 5,
        SessionPhase::Closed => 6,
    }
}

impl FrameSampler {
    fn observe(
        &mut self,
        scope: Option<SamplingScope>,
        seconds: f64,
        phase: SessionPhase,
    ) -> Option<FrameReport> {
        if self.scope != scope {
            self.scope = scope;
            self.settle_remaining = SETTLE_SECONDS;
            self.count = 0;
            self.phases = [0; 7];
            return None;
        }
        if scope.is_none() || !seconds.is_finite() || seconds <= 0.0 {
            return None;
        }
        if self.settle_remaining > 0.0 {
            self.settle_remaining = (self.settle_remaining - seconds).max(0.0);
            return None;
        }
        self.intervals[self.count] = seconds;
        self.count += 1;
        self.phases[phase_index(phase)] += 1;
        if self.count != SAMPLE_COUNT {
            return None;
        }

        // Fixed storage and an allocation-free sort, only once per window.
        let elapsed_seconds = self.intervals.iter().sum::<f64>();
        self.intervals.sort_unstable_by(f64::total_cmp);
        let report = FrameReport {
            median_ms: (self.intervals[SAMPLE_COUNT / 2 - 1] + self.intervals[SAMPLE_COUNT / 2])
                * 500.0,
            // Nearest-rank percentile: rank ceil(0.95 * 300) = 285.
            p95_ms: self.intervals[(SAMPLE_COUNT * 95).div_ceil(100) - 1] * 1000.0,
            fps: 300.0 / elapsed_seconds,
            elapsed_seconds,
            phases: self.phases,
        };
        self.count = 0;
        self.phases = [0; 7];
        Some(report)
    }
}

// Bevy injects independent timing, view and window resources into the sampler.
#[allow(clippy::too_many_arguments)]
fn sample_frames(
    time: Res<Time<Real>>,
    view: Res<PrimaryView>,
    navigation: Res<ProductionNavigation>,
    session: Res<ObserverSession>,
    ui: Res<ObserverUiState>,
    viewport: Res<ObserverViewport>,
    scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut sampler: ResMut<FrameSampler>,
) {
    let scope = windows
        .single()
        .ok()
        .zip(viewport.0)
        .and_then(|(window, rect)| {
            if !window.focused
                || ui.menu_open
                || ui.splash_visible
                || ui.comparison_open
                || ui.disclosure.is_some()
                || rect.width() <= 0.0
                || rect.height() <= 0.0
            {
                return None;
            }
            Some(SamplingScope {
                mode: match (*view, navigation.flat) {
                    (PrimaryView::Map, _) => RenderMode::Map3d,
                    (PrimaryView::Production, false) => RenderMode::Production3d,
                    (PrimaryView::Production, true) => RenderMode::Production2d,
                },
                campaign: session.campaign,
                perspective: session.perspective,
                window_pixels: UVec2::new(window.physical_width(), window.physical_height()),
                viewport_origin: (rect.min * window.scale_factor()).as_uvec2(),
                viewport_pixels: (rect.size() * window.scale_factor()).as_uvec2(),
                ui_scale_bits: scale.0.to_bits(),
                history_open: ui.history_open,
                archive_open: ui.archive_open,
                details_open: navigation.details_open,
                reduced_motion: ui.reduced_motion,
                playing: session.playing,
            })
        });
    let Some(report) = sampler.observe(scope, time.delta_secs_f64(), session.phase) else {
        return;
    };
    let Some(scope) = scope else {
        return;
    };
    let perspective = match scope.perspective {
        Perspective::FullObserver => "observer",
        Perspective::PlayerKnowledge => "known",
    };
    log::info!(
        "frame_perf view={} perspective={} window={}x{} viewport={}x{}+{}+{} ui_scale={:.2} history={} archive={} details={} reduced_motion={} playing={} samples={} elapsed_s={:.3} median_ms={:.3} p95_ms={:.3} fps={:.2} phase_frames=connecting:{},loading:{},ready:{},advancing:{},complete:{},failed:{},closed:{}",
        scope.mode.label(), perspective,
        scope.window_pixels.x, scope.window_pixels.y,
        scope.viewport_pixels.x, scope.viewport_pixels.y, scope.viewport_origin.x, scope.viewport_origin.y,
        f32::from_bits(scope.ui_scale_bits), scope.history_open, scope.archive_open, scope.details_open, scope.reduced_motion, scope.playing,
        SAMPLE_COUNT, report.elapsed_seconds, report.median_ms, report.p95_ms, report.fps,
        report.phases[0], report.phases[1], report.phases[2], report.phases[3],
        report.phases[4], report.phases[5], report.phases[6],
    );
}

/// Logs complete, settled native frame windows through the existing file sink.
pub struct ObserverPerformancePlugin;

impl Plugin for ObserverPerformancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameSampler>()
            .add_systems(Last, sample_frames);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(mode: RenderMode) -> SamplingScope {
        SamplingScope {
            mode,
            campaign: CampaignId::from_uuid(uuid::Uuid::nil()),
            perspective: Perspective::FullObserver,
            window_pixels: UVec2::new(1366, 768),
            viewport_origin: UVec2::new(16, 96),
            viewport_pixels: UVec2::new(949, 282),
            ui_scale_bits: 1.0_f32.to_bits(),
            history_open: true,
            archive_open: false,
            details_open: false,
            reduced_motion: false,
            playing: false,
        }
    }

    fn settle(sampler: &mut FrameSampler, scope: SamplingScope) {
        assert!(sampler
            .observe(Some(scope), 0.5, SessionPhase::Ready)
            .is_none());
        for _ in 0..4 {
            assert!(sampler
                .observe(Some(scope), 0.5, SessionPhase::Ready)
                .is_none());
        }
        assert_eq!(sampler.count, 0);
    }

    #[test]
    fn real_intervals_produce_exact_median_nearest_rank_p95_and_phase_counts() {
        let mut sampler = FrameSampler::default();
        let scope = scope(RenderMode::Production3d);
        settle(&mut sampler, scope);
        let mut report = None;
        for index in 0..SAMPLE_COUNT {
            let (seconds, phase) = match index {
                0..150 => (0.010, SessionPhase::Ready),
                150..285 => (0.020, SessionPhase::Loading),
                _ => (0.030, SessionPhase::Advancing),
            };
            report = sampler.observe(Some(scope), seconds, phase);
            assert_eq!(report.is_some(), index == SAMPLE_COUNT - 1);
        }
        let report = report.expect("complete native frame window");
        assert!((report.median_ms - 15.0).abs() < 1e-9);
        assert!((report.p95_ms - 20.0).abs() < 1e-9);
        assert!((report.elapsed_seconds - 4.65).abs() < 1e-9);
        assert!((report.fps - 300.0 / 4.65).abs() < 1e-9);
        assert_eq!(report.phases, [0, 135, 150, 15, 0, 0, 0]);
        assert_eq!(sampler.count, 0);
        assert_eq!(sampler.phases, [0; 7]);
        assert!(sampler
            .observe(Some(scope), 0.010, SessionPhase::Ready)
            .is_none());
        assert_eq!(
            sampler.count, 1,
            "consecutive windows do not repeat settling"
        );
    }

    #[test]
    fn view_menu_resolution_and_perspective_boundaries_discard_partial_windows() {
        let mut sampler = FrameSampler::default();
        let mut current = scope(RenderMode::Map3d);
        settle(&mut sampler, current);
        sampler.observe(Some(current), 0.010, SessionPhase::Ready);
        assert_eq!(sampler.count, 1);
        current.mode = RenderMode::Production2d;
        settle(&mut sampler, current);
        sampler.observe(Some(current), 0.010, SessionPhase::Ready);
        sampler.observe(None, 1.0, SessionPhase::Ready);
        assert_eq!(sampler.count, 0);
        settle(&mut sampler, current);
        current.window_pixels = UVec2::new(1920, 1080);
        settle(&mut sampler, current);
        current.perspective = Perspective::PlayerKnowledge;
        settle(&mut sampler, current);
        sampler.observe(Some(current), 0.010, SessionPhase::Ready);
        current.details_open = true;
        settle(&mut sampler, current);
    }

    #[test]
    fn invalid_intervals_are_not_samples_and_real_stalls_are_preserved() {
        let mut sampler = FrameSampler::default();
        let scope = scope(RenderMode::Production3d);
        settle(&mut sampler, scope);
        for seconds in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(sampler
                .observe(Some(scope), seconds, SessionPhase::Ready)
                .is_none());
        }
        assert_eq!(sampler.count, 0);
        sampler.observe(Some(scope), 3.0, SessionPhase::Advancing);
        assert_eq!(sampler.intervals[0].to_bits(), 3.0_f64.to_bits());
        assert_eq!(sampler.phases[phase_index(SessionPhase::Advancing)], 1);
    }
}
