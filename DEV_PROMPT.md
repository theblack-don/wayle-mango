# Wayle MangoWM Fork — Development Prompt

## Context

This is a fork of Wayle (https://github.com/wayle-rs/wayle) to add MangoWM support and fix a theming bug that prevents module color overrides from working when using dynamic color providers (Matugen, Pywal, Wallust).

## Issue 1: Module Color Overrides Ignored with Non-Wayle Theme Providers

### Problem

When `theme-provider` is set to `"matugen"`, `"pywal"`, or `"wallust"`, user-configured module colors (e.g., `icon-bg-color`, `label-color`, `border-color`) in `runtime.toml` or `config.toml` are **completely ignored**. The bar buttons always fall back to their compiled-in defaults:

- `volume` → `red` (pink)
- `microphone` → `red` (pink)
- `power` → `red` (pink)
- `dashboard` → `yellow`
- `network` → `accent` (coincidentally works because its default is `accent`)

### Root Cause

In `crates/wayle-widgets/src/styling.rs`, the `resolve_color()` function has this logic:

```rust
pub fn resolve_color(prop: &ConfigProperty<ColorValue>, is_wayle_theme: bool) -> Cow<'static, str> {
    if is_wayle_theme {
        prop.get().to_css()      // respects user overrides
    } else {
        prop.default().to_css()  // ignores user overrides!
    }
}
```

When `theme-provider != "wayle"`, `is_wayle_theme` is `false`, so ALL user color overrides are discarded and replaced with the hardcoded default colors.

This affects every bar button module because `BarButton::build_css()` in `crates/wayle-widgets/src/components/bar_buttons/styling.rs` calls `resolve_color()` for all color slots.

### Files Involved

- `crates/wayle-widgets/src/styling.rs` — `resolve_color()` function (the bug)
- `crates/wayle-widgets/src/components/bar_buttons/styling.rs` — `build_css()` calls `resolve_color()`
- `crates/wayle-widgets/src/components/bar_buttons/component.rs` — `resolve_icon_color()` has similar `is_wayle_theme` logic

### Proposed Fix

The `resolve_color()` function should **always** respect user overrides. The `is_wayle_theme` check was intended to prevent clashing with external GTK themes, but it incorrectly also discards user-configured color tokens (like `"accent"`, `"red"`, `"fg-muted"`) which are perfectly valid regardless of theme provider.

**Option A (Recommended):** Remove the `is_wayle_theme` check from `resolve_color()` entirely. Always use `prop.get()`. If users want to avoid clashing with external GTK themes, they can simply not set custom colors.

**Option B:** Only fall back to defaults for `ColorValue::Custom(hex)` (fixed hex colors), but still respect `ColorValue::Token(...)` and `ColorValue::Auto` overrides, since tokens are designed to work with any theme provider.

### Current Workaround

Users can override CSS custom properties in `~/.config/wayle/styles/index.scss`:

```scss
.volume .bar-button,
.microphone .bar-button,
.dashboard .bar-button {
  --bar-btn-icon-bg: var(--accent);
  --bar-btn-label-color: var(--accent);
  --bar-btn-border-color: var(--accent);
}

.power .bar-button {
  --bar-btn-icon-bg: var(--accent-subtle);
  --bar-btn-icon-color: var(--accent);
  --bar-btn-border-color: var(--accent);
}
```

---

## Issue 2: Add MangoWM Workspace Switcher Module

### Goal

Create a `mango-workspaces` bar module that displays MangoWM workspaces and allows click-to-switch, following the same pattern as `niri_workspaces` and `hyprland_workspaces`.

### Investigation Needed

1. **MangoWM IPC mechanism**: How does MangoWM expose workspace state? Does it use:
   - D-Bus (like Niri uses `NIRI_SOCKET` but also has a D-Bus service?)
   - Unix socket (like Hyprland's `HYPRLAND_INSTANCE_SIGNATURE` + socket)
   - Wayland protocol extensions
   - A custom `wayle-mango` IPC crate needs to be created if one doesn't exist

2. Check if MangoWM sets any environment variables (e.g., `MANGO_SOCKET`, `XDG_CURRENT_DESKTOP=Mango`) for compositor detection.

### Files to Create/Modify

**New files (follow `niri_workspaces` pattern):**
- `crates/wayle-shell/src/shell/bar/modules/mango_workspaces/mod.rs`
- `crates/wayle-shell/src/shell/bar/modules/mango_workspaces/factory.rs`
- `crates/wayle-shell/src/shell/bar/modules/mango_workspaces/messages.rs`
- `crates/wayle-shell/src/shell/bar/modules/mango_workspaces/watchers.rs`
- `crates/wayle-shell/src/shell/bar/modules/mango_workspaces/helpers.rs`
- `crates/wayle-shell/src/shell/bar/modules/mango_workspaces/styling.rs`
- `crates/wayle-shell/src/shell/bar/modules/mango_workspaces/filtering.rs`
- `crates/wayle-shell/src/shell/bar/modules/mango_workspaces/button/mod.rs`
- `crates/wayle-shell/src/shell/bar/modules/mango_workspaces/button/methods.rs`

**Config schema:**
- `crates/wayle-config/src/schemas/modules/mango_workspaces/mod.rs` — follow `NiriWorkspacesConfig` pattern
- `crates/wayle-config/src/schemas/modules/mod.rs` — add `pub mod mango_workspaces`
- `crates/wayle-config/src/schemas/modules/mod.rs` — add to `ModulesConfig`

**Bar module registry:**
- `crates/wayle-config/src/schemas/bar/types/mod.rs` — add `MangoWorkspaces` to `BarModule` enum
- `crates/wayle-shell/src/shell/bar/modules/mod.rs` — add to `register_modules!` macro

**Compositor detection:**
- `crates/wayle-shell/src/shell/bar/modules/compositor.rs` — add `Mango` variant and detection logic

**Services:**
- `crates/wayle-shell/src/shell/services.rs` — add `mango: Option<Arc<MangoService>>` (if needed)
- May need to create a new `crates/wayle-mango` IPC crate (follow `wayle-hyprland` and `wayle-niri` patterns)

---

## Issue 3: Make Window Title Module Work with MangoWM

### Problem

The `window_title` module currently logs:
```
unsupported compositor module="window-title" compositor=mango
```

### Root Cause

`window_title/factory.rs` detects compositors and only supports Hyprland and Niri:

```rust
fn build_source(services: &ShellServices) -> Option<Arc<dyn FocusedWindowSource>> {
    match Compositor::detect() {
        Compositor::Hyprland => { ... }
        Compositor::Niri => { ... }
        Compositor::Unknown(name) => {
            warn!(module = "window-title", compositor = %name, "unsupported compositor");
            None
        }
    }
}
```

### Files to Modify

- `crates/wayle-shell/src/shell/bar/modules/compositor.rs` — detect MangoWM
- `crates/wayle-shell/src/shell/bar/modules/window_title/factory.rs` — add Mango branch
- `crates/wayle-shell/src/shell/bar/modules/window_title/sources/` — create `MangoFocusedWindowSource` (follow `NiriFocusedWindowSource` / `HyprlandFocusedWindowSource` pattern)

### Investigation Needed

- Does MangoWM expose focused window info via the same IPC mechanism as workspaces?
- What fields are available? (title, app_id, class, etc.)
- How to subscribe to focus changes?

---

## Additional Notes

### Compositor Detection Pattern

Current detection in `compositor.rs`:
```rust
pub(crate) fn detect() -> Self {
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return Self::Hyprland;
    }
    if env::var("NIRI_SOCKET").is_ok() {
        return Self::Niri;
    }
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    Self::Unknown(desktop)
}
```

For MangoWM, you may need to check `XDG_CURRENT_DESKTOP` for a Mango-specific value, or a `MANGO_` environment variable.

### Existing Workspace Module Architecture

Both `niri_workspaces` and `hyprland_workspaces` follow this structure:
- `mod.rs` — Main Relm4 component, renders workspace buttons
- `factory.rs` — `ModuleFactory` impl, creates the component
- `messages.rs` — `Cmd` and `Msg` enums
- `watchers.rs` — Spawns watchers for workspace state changes
- `helpers.rs` — Workspace sorting, label formatting
- `filtering.rs` — Filtering visible/hidden/empty workspaces
- `styling.rs` — CSS variable injection for workspace colors
- `button/` — Individual workspace button subcomponent

The Mango module should reuse as much as possible from the existing workspace types in `crates/wayle-config/src/schemas/modules/` (e.g., `WorkspaceStyle`, `ActiveIndicator`, `DisplayMode`, `LabelStrategy`, etc.).

### Dependencies

Check if MangoWM has an IPC library or protocol spec. If not, you may need to:
1. Create `crates/wayle-mango` (follow `wayle-hyprland` and `wayle-niri` crate patterns)
2. Or use raw socket/D-Bus communication directly in the shell module

### Testing

After making changes, test with:
```bash
cargo build --release
# Restart wayle shell
killall wayle && walye panel start &
```

For the color fix specifically, test by setting:
```toml
[styling]
theme-provider = "matugen"

[modules.volume]
icon-bg-color = "accent"
label-color = "accent"
```

The volume button should show the wallpaper's primary accent color without needing any SCSS workaround.
