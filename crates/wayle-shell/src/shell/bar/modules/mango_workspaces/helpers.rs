//! Pure helpers for Mango tag rendering and CSS class naming.

use std::collections::HashMap;

use wayle_config::schemas::modules::TagStyle;

/// CSS class used to address a tag button by index, e.g. `tag-id-3`.
pub(super) fn tag_id_css_class(index: u32) -> String {
    format!("tag-id-{index}")
}

/// Looks up a per-tag style override by 1-based index.
pub(super) fn tag_style(index: u32, map: &HashMap<u32, TagStyle>) -> Option<&TagStyle> {
    map.get(&index)
}

/// Returns `true` when the tag index matches any of the ignore patterns.
pub(super) fn is_ignored(index: u32, patterns: &[String]) -> bool {
    let idx_str = index.to_string();
    patterns.iter().any(|pattern| crate::glob::matches(pattern, &idx_str))
}
