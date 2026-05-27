//! [`MangoTagButton`] private helpers: state-driven rendering decisions.

use relm4::gtk;
use wayle_config::schemas::modules::MangoDisplayMode;

use super::{MangoTagButton, MangoTagButtonInit};
use crate::shell::bar::modules::mango_workspaces::helpers;

impl MangoTagButton {
    pub(super) fn show_label(&self) -> bool {
        let has_label = self.label.as_deref().is_some_and(|label| !label.is_empty());
        if !has_label {
            return false;
        }
        matches!(self.display_mode, MangoDisplayMode::Label)
            || (matches!(self.display_mode, MangoDisplayMode::Icon) && self.icon.is_none())
    }

    pub(super) fn show_icon(&self) -> bool {
        matches!(self.display_mode, MangoDisplayMode::Icon) && self.icon.is_some()
    }

    pub(super) fn label_text(&self) -> &str {
        self.label.as_deref().unwrap_or("")
    }

    pub(super) fn orientation(&self) -> gtk::Orientation {
        if self.is_vertical {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        }
    }

    pub(super) fn content_halign(&self) -> gtk::Align {
        if self.is_vertical {
            gtk::Align::Fill
        } else {
            gtk::Align::Center
        }
    }

    pub(super) fn content_valign(&self) -> gtk::Align {
        if self.is_vertical {
            gtk::Align::Center
        } else {
            gtk::Align::Fill
        }
    }
}

pub(super) fn compute_css_classes(init: &MangoTagButtonInit) -> Vec<String> {
    let mut classes = vec![String::from("workspace")];

    let state = if init.is_active {
        "active"
    } else if init.has_clients {
        "occupied"
    } else {
        "empty"
    };
    classes.push(state.to_string());

    if init.is_urgent {
        classes.push(String::from("urgent"));
    }

    classes.push(init.active_indicator.css_class().to_string());

    if init.is_vertical {
        classes.push(String::from("vertical"));
    }

    classes.push(helpers::tag_id_css_class(init.index));

    classes
}
