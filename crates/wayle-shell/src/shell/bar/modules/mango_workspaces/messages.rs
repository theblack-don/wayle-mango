//! Init parameters, input messages, and command outputs for the
//! [`MangoWorkspaces`] Relm4 component.

use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;
use crate::services::MangoService;

pub(crate) struct MangoWorkspacesInit {
    pub settings: BarSettings,
    pub mango: Arc<MangoService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum MangoWorkspacesMsg {
    TagClicked(u32),
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum MangoWorkspacesCmd {
    TagsChanged,
    ConfigChanged,
}
