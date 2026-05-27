//! Mango workspaces module settings.

use wayle_config::Config;

use crate::{
    editors::{
        color_value::color_value,
        enum_select::enum_select,
        number::{number_u8, scale, spacing},
        toggle::toggle,
        toml_editor::toml_editor,
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.mango_workspaces;

    LeafEntry {
        id: "mango-workspaces",
        i18n_key: "settings-nav-mango-workspaces",
        icon: "ld-grid-2x2-symbolic",
        spec: page_spec(
            "settings-page-mango-workspaces",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        number_u8(&module.min_tag_count),
                        toggle(&module.monitor_specific),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![
                        enum_select(&module.display_mode),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-sizing",
                    items: vec![
                        spacing(&module.tag_padding),
                        scale(&module.icon_size),
                        scale(&module.label_size),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-mappings",
                    items: vec![
                        toml_editor(
                            &module.tag_map,
                            "tag-map",
                            &config.styling.palette.bg,
                        ),
                        toml_editor(
                            &module.tag_ignore,
                            "tag-ignore",
                            &config.styling.palette.bg,
                        ),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![toggle(&module.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        enum_select(&module.active_indicator),
                        color_value(&module.active_color),
                        color_value(&module.occupied_color),
                        color_value(&module.empty_color),
                        color_value(&module.container_bg_color),
                        color_value(&module.border_color),
                    ],
                },
            ],
        ),
    }
}
