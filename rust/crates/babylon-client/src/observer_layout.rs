//! Shared geometry: a large world and one mutually exclusive subject rail.

use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub(crate) enum ObserverRegion {
    Context,
    Log,
    History,
    Footer,
}

/// Logical UI coordinates. Context, Archive, readings and delivery history
/// share the right rail; their existing visibility and focus gates select its owner.
pub(crate) struct ObserverLayout {
    pub world: Rect,
    subject: Rect,
    history: Rect,
    footer: Rect,
}

impl ObserverLayout {
    pub fn new(window_size: Vec2, scale: f32, history_open: bool) -> Self {
        let size = window_size / scale;
        let sidebar_width = (size.x * 0.27).clamp(300.0, 380.0);
        let world_right = (size.x - sidebar_width - 32.0).max(200.0);
        let world_bottom = size.y - 64.0;
        let history_top = (world_bottom - 180.0).max(164.0);
        Self {
            world: Rect::new(
                16.0,
                84.0,
                world_right,
                if history_open {
                    history_top - 12.0
                } else {
                    world_bottom
                },
            ),
            subject: Rect::new(world_right + 16.0, 84.0, size.x - 16.0, size.y - 16.0),
            history: Rect::new(16.0, history_top, world_right, world_bottom),
            footer: Rect::new(16.0, size.y - 52.0, world_right, size.y - 12.0),
        }
    }

    pub fn region(&self, region: ObserverRegion) -> Rect {
        match region {
            ObserverRegion::Context | ObserverRegion::Log => self.subject,
            ObserverRegion::History => self.history,
            ObserverRegion::Footer => self.footer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_and_active_panels_remain_disjoint_at_supported_sizes_and_scales() {
        for size in [Vec2::new(1366.0, 768.0), Vec2::new(1920.0, 1080.0)] {
            for scale in [1.0, 1.15, 1.3] {
                for history_open in [false, true] {
                    let layout = ObserverLayout::new(size, scale, history_open);
                    let subject = layout.region(ObserverRegion::Context);
                    assert_eq!(subject, layout.region(ObserverRegion::Log));
                    assert!(layout.world.width() > 600.0);
                    assert!(layout.world.height() > 200.0);
                    assert!(layout.world.max.x < subject.min.x);
                    assert!(layout.world.max.y < layout.footer.min.y);
                    assert!(subject.height() > size.y / scale * 0.78);
                    if history_open {
                        assert!(layout.world.max.y < layout.history.min.y);
                        assert!(layout.history.max.y < layout.footer.min.y);
                        assert!(layout.history.max.x < subject.min.x);
                    } else {
                        assert!(layout.world.height() > size.y / scale * 0.74);
                    }
                    for region in [layout.world, subject, layout.history, layout.footer] {
                        assert!(region.min.x >= 0.0 && region.min.y >= 0.0);
                        assert!(region.max.x <= size.x / scale);
                        assert!(region.max.y <= size.y / scale);
                    }
                }
            }
        }
    }

    #[test]
    fn history_only_borrows_world_height_and_does_not_shift_the_selected_subject() {
        let size = Vec2::new(1366.0, 768.0);
        let closed = ObserverLayout::new(size, 1.15, false);
        let open = ObserverLayout::new(size, 1.15, true);
        assert_eq!(closed.subject, open.subject);
        assert_eq!(closed.footer, open.footer);
        assert_eq!(closed.world.min, open.world.min);
        assert!(closed.world.height() > open.world.height());
    }
}
