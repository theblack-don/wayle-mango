//! MangoWM workspace (tag) switcher configuration.

use std::{collections::HashMap, ops::Deref};

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wayle_derive::{wayle_config, wayle_enum};

use crate::{
    ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ScaleFactor, Spacing},
};

/// What identifies each tag button.
#[wayle_enum(default)]
pub enum DisplayMode {
    /// Show tag number.
    #[default]
    Label,
    /// Show icon from `tag-map` (falls back to label if unmapped).
    Icon,
    /// Show nothing.
    None,
}

/// Visual indicator style for the active tag.
#[wayle_enum(default)]
pub enum ActiveIndicator {
    /// Entire button gets a colored background.
    #[default]
    Background,
    /// Small colored bar under the tag button.
    Underline,
}

impl ActiveIndicator {
    /// CSS class for this indicator style.
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Background => "indicator-background",
            Self::Underline => "indicator-underline",
        }
    }
}

/// Per-tag styling override.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TagStyle {
    /// Custom icon for this tag.
    pub icon: Option<String>,
    /// Custom background color for this tag when active.
    pub color: Option<ColorValue>,
}

/// Per-tag icon and color overrides, keyed by tag index (1-based string).
#[derive(Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(transparent)]
pub struct TagMap(HashMap<u32, TagStyle>);

impl Deref for TagMap {
    type Target = HashMap<u32, TagStyle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a TagMap {
    type Item = (&'a u32, &'a TagStyle);
    type IntoIter = std::collections::hash_map::Iter<'a, u32, TagStyle>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Serialize for TagMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, &TagStyle> = self
            .0
            .iter()
            .map(|(key, val)| (key.to_string(), val))
            .collect();
        string_map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TagMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, TagStyle> = HashMap::deserialize(deserializer)?;
        let mut result = HashMap::with_capacity(string_map.len());
        for (key, value) in string_map {
            let id: u32 = key.parse().map_err(serde::de::Error::custom)?;
            result.insert(id, value);
        }
        Ok(TagMap(result))
    }
}

/// MangoWM tag (workspace) switcher.
#[wayle_config(i18n_prefix = "settings-modules-mango-workspaces")]
pub struct MangoWorkspacesConfig {
    /// Minimum number of tag buttons to display.
    ///
    /// When set to 0 (default), only active and occupied tags are shown.
    /// When set to N, at least N buttons are always visible.
    #[serde(rename = "min-tag-count")]
    #[default(0)]
    pub min_tag_count: ConfigProperty<u8>,

    /// Show only tags belonging to the bar's monitor.
    ///
    /// When true, each bar shows only its monitor's tags.
    #[serde(rename = "monitor-specific")]
    #[default(true)]
    pub monitor_specific: ConfigProperty<bool>,

    /// What identifies each tag button.
    #[serde(rename = "display-mode")]
    #[default(DisplayMode::Label)]
    pub display_mode: ConfigProperty<DisplayMode>,

    /// Visual indicator for the active tag.
    #[serde(rename = "active-indicator")]
    #[default(ActiveIndicator::Background)]
    pub active_indicator: ConfigProperty<ActiveIndicator>,

    /// Padding for tag content along the bar direction.
    #[serde(rename = "tag-padding")]
    #[default(Spacing::new(0.5))]
    pub tag_padding: ConfigProperty<Spacing>,

    /// Scale multiplier for tag icons. Range: 0.25-3.0.
    #[serde(rename = "icon-size")]
    #[default(ScaleFactor::default())]
    pub icon_size: ConfigProperty<ScaleFactor>,

    /// Scale multiplier for tag labels. Range: 0.25-3.0.
    #[serde(rename = "label-size")]
    #[default(ScaleFactor::default())]
    pub label_size: ConfigProperty<ScaleFactor>,

    /// Tags to hide from the display.
    ///
    /// Glob patterns matched against the tag index.
    #[serde(rename = "tag-ignore")]
    #[default(Vec::new())]
    pub tag_ignore: ConfigProperty<Vec<String>>,

    /// Color for the active (focused) tag.
    #[serde(rename = "active-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
    pub active_color: ConfigProperty<ColorValue>,

    /// Color for occupied tags (have clients but not focused).
    #[serde(rename = "occupied-color")]
    #[default(ColorValue::Token(CssToken::FgMuted))]
    pub occupied_color: ConfigProperty<ColorValue>,

    /// Color for empty tags.
    #[serde(rename = "empty-color")]
    #[default(ColorValue::Token(CssToken::FgSubtle))]
    pub empty_color: ConfigProperty<ColorValue>,

    /// Background color for the tags container.
    #[serde(rename = "container-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub container_bg_color: ConfigProperty<ColorValue>,

    /// Display border around the tags container.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color for the tags container.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::BorderDefault))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Per-tag icon and color overrides, keyed by tag index.
    ///
    /// ## Example
    ///
    /// ```toml
    /// [modules.mango-workspaces.tag-map]
    /// 1 = { icon = "ld-globe-symbolic", color = "#4a90d9" }
    /// 2 = { icon = "ld-terminal-symbolic" }
    /// ```
    #[serde(rename = "tag-map")]
    #[default(TagMap::default())]
    pub tag_map: ConfigProperty<TagMap>,
}

impl ModuleInfoProvider for MangoWorkspacesConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("mango-workspaces"),
            schema: || schema_for!(MangoWorkspacesConfig),
            layout_id: Some(String::from("mango-workspaces")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::standard()
    }
}

crate::register_module!(MangoWorkspacesConfig);
