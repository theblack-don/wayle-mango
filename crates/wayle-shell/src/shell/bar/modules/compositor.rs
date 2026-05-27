//! Compositor detection for compositor-dependent modules.

use std::env;

/// Detected Wayland compositor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Compositor {
    /// Hyprland compositor.
    Hyprland,
    /// MangoWM compositor.
    Mango,
    /// niri compositor.
    Niri,
    /// Unknown or unsupported compositor.
    Unknown(String),
}

impl Compositor {
    /// Detects the running Wayland compositor.
    pub(crate) fn detect() -> Self {
        if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            return Self::Hyprland;
        }

        if env::var("NIRI_SOCKET").is_ok() {
            return Self::Niri;
        }

        if env::var("MANGO_INSTANCE_SIGNATURE").is_ok() {
            return Self::Mango;
        }

        let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        if desktop.eq_ignore_ascii_case("mango") {
            return Self::Mango;
        }

        Self::Unknown(desktop)
    }
}
