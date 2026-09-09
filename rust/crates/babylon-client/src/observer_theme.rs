//! Deliberate surface, selection, and relationship colors for the observer.
use bevy::prelude::Color;

pub const INK: Color = Color::srgb_u8(17, 21, 27);
pub const PANEL: Color = Color::srgb_u8(32, 41, 50);
pub const PAPER: Color = Color::srgb_u8(241, 234, 217);
pub const YELLOW: Color = Color::srgb_u8(216, 189, 99);
pub const RED: Color = Color::srgb_u8(241, 91, 76);
pub const GRAY: Color = Color::srgb_u8(186, 194, 200);
pub const BLUE: Color = Color::srgb_u8(84, 188, 212);
/// Geographic surface only: this color encodes no economic measurement.
pub const LAND: Color = Color::srgb_u8(56, 66, 74);
/// Outgoing supply, paired with a directional arrow and an explicit label.
pub const COPPER: Color = Color::srgb_u8(234, 161, 108);
