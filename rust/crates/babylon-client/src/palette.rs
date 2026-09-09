//! KSBC role colors (`DESIGN_BIBLE` §9b). Source of truth:
//! `src/babylon/render/tiers.py::TRUECOLOR_PALETTE`; the parity guard
//! `tests/unit/render/test_rust_theme_parity.py` parses THIS file's
//! `Color::srgb_u8(r, g, b)` literals — keep each constant on one line.
//!
//! B0's scaffold only draws FIELD (clear color) and GOLD (title text); the
//! rest of the canonical role palette is declared here now so the parity
//! guard has a complete table to check from day one — later milestones
//! (map tiles, UI chrome) consume them. `dead_code` is allowed for that
//! reason, not to paper over an unused mistake.
#![allow(dead_code)]
use bevy::color::Color;

pub const FIELD: Color = Color::srgb_u8(26, 0, 0);
pub const BONE: Color = Color::srgb_u8(232, 232, 232);
pub const CRIMSON: Color = Color::srgb_u8(220, 20, 60);
pub const GOLD: Color = Color::srgb_u8(255, 215, 0);
pub const DIM: Color = Color::srgb_u8(64, 64, 64);
pub const MUTED_DARK: Color = Color::srgb_u8(32, 32, 32);
pub const ROYAL: Color = Color::srgb_u8(65, 105, 225);
pub const GREEN_DARK: Color = Color::srgb_u8(34, 139, 34);
