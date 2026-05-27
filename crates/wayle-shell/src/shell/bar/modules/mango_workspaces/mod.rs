//! MangoWM tag (workspace) switcher bar module.

mod button;
mod factory;
mod helpers;
mod messages;
mod styling;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, prelude::*};
use wayle_config::ConfigService;
use wayle_widgets::{prelude::BarSettings, utils::force_window_resize};

use self::button::{MangoTagButton, MangoTagButtonInit, MangoTagButtonOutput};
pub(crate) use self::{
    factory::Factory,
    messages::{MangoWorkspacesCmd, MangoWorkspacesInit, MangoWorkspacesMsg},
};
use crate::shell::{bar::dropdowns::DropdownRegistry, helpers::COMPONENT_CSS_PRIORITY};

pub(crate) struct MangoWorkspaces {
    pub(super) mango: Arc<crate::services::MangoService>,
    pub(super) config: Arc<ConfigService>,
    pub(super) settings: BarSettings,
    #[allow(dead_code)]
    pub(super) dropdowns: Rc<DropdownRegistry>,
    pub(super) css_provider: gtk::CssProvider,
    pub(super) buttons: FactoryVecDeque<MangoTagButton>,
}

#[relm4::component(pub(crate))]
impl Component for MangoWorkspaces {
    type Init = MangoWorkspacesInit;
    type Input = MangoWorkspacesMsg;
    type Output = ();
    type CommandOutput = MangoWorkspacesCmd;

    view! {
        gtk::Box {
            add_css_class: "workspaces",
            add_css_class: "mango",
            #[watch]
            set_orientation: model.orientation(),
            #[watch]
            set_hexpand: model.is_vertical(),
            #[watch]
            set_vexpand: !model.is_vertical(),
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let workspaces_config = &config.modules.mango_workspaces;
        let theme_provider = config.styling.theme_provider.clone();
        let bar_scale = config.bar.scale.clone();

        watchers::spawn_watchers(
            &sender,
            workspaces_config,
            init.mango.clone(),
            theme_provider,
            bar_scale,
            &init.settings,
        );

        let css_provider = gtk::CssProvider::new();
        gtk::style_context_add_provider_for_display(
            &root.display(),
            &css_provider,
            COMPONENT_CSS_PRIORITY,
        );

        let buttons = FactoryVecDeque::builder().launch(root.clone()).forward(
            sender.input_sender(),
            |output| match output {
                MangoTagButtonOutput::Clicked(index) => MangoWorkspacesMsg::TagClicked(index),
                MangoTagButtonOutput::ScrollUp => MangoWorkspacesMsg::ScrollUp,
                MangoTagButtonOutput::ScrollDown => MangoWorkspacesMsg::ScrollDown,
            },
        );

        let mut model = Self {
            mango: init.mango,
            config: init.config,
            settings: init.settings,
            dropdowns: init.dropdowns,
            css_provider,
            buttons,
        };
        styling::apply_styling(&model.css_provider, &model.config, &model.settings);
        model.rebuild_buttons();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            MangoWorkspacesMsg::TagClicked(index) => {
                self.switch_to_tag(index);
            }
            MangoWorkspacesMsg::ScrollUp => {
                self.navigate_tag(-1);
            }
            MangoWorkspacesMsg::ScrollDown => {
                self.navigate_tag(1);
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: MangoWorkspacesCmd,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            MangoWorkspacesCmd::TagsChanged => {
                self.rebuild_buttons();
                force_window_resize(root);
            }
            MangoWorkspacesCmd::ConfigChanged => {
                styling::apply_styling(&self.css_provider, &self.config, &self.settings);
                self.rebuild_buttons();
                force_window_resize(root);
            }
        }
    }
}

impl Drop for MangoWorkspaces {
    fn drop(&mut self) {
        gtk::style_context_remove_provider_for_display(
            &self.buttons.widget().display(),
            &self.css_provider,
        );
    }
}

impl MangoWorkspaces {
    pub(super) fn is_vertical(&self) -> bool {
        self.settings.is_vertical.get()
    }

    pub(super) fn orientation(&self) -> gtk::Orientation {
        if self.is_vertical() {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        }
    }

    pub(super) fn rebuild_buttons(&mut self) {
        let monitor = self.settings.monitor_name.clone().unwrap_or_default();
        let runtime = tokio::runtime::Handle::current();
        let tags_result = runtime.block_on(self.mango.get_tags(&monitor));

        let tags = match tags_result {
            Ok(t) => t.tags,
            Err(_) => return,
        };

        let config = self.config.config();
        let ws_config = &config.modules.mango_workspaces;
        let display_mode = ws_config.display_mode.get();
        let active_indicator = ws_config.active_indicator.get();
        let is_vertical = self.is_vertical();
        let ignore_patterns = ws_config.tag_ignore.get();
        let min_tag_count = ws_config.min_tag_count.get() as usize;
        let tag_map = ws_config.tag_map.get();

        let mut displayed: Vec<_> = tags
            .into_iter()
            .filter(|tag| !helpers::is_ignored(tag.index, &ignore_patterns))
            .collect();

        // Ensure minimum tag count
        let max_index = displayed.iter().map(|t| t.index).max().unwrap_or(0);
        let needed = min_tag_count.saturating_sub(displayed.len());
        for i in 1..=needed as u32 {
            let index = max_index + i;
            displayed.push(crate::services::MangoTag {
                index,
                is_active: false,
                is_urgent: false,
                layout: String::new(),
                client_count: 0,
            });
        }

        // Sort by index
        displayed.sort_by_key(|tag| tag.index);

        {
            let mut guard = self.buttons.guard();
            guard.clear();
            for tag in displayed {
                let label = Some(tag.index.to_string());
                let icon = helpers::tag_style(tag.index, &tag_map).and_then(|s| s.icon.clone());

                let init = MangoTagButtonInit {
                    index: tag.index,
                    label,
                    icon,
                    is_active: tag.is_active,
                    is_urgent: tag.is_urgent,
                    has_clients: tag.client_count > 0,
                    is_vertical,
                    display_mode,
                    active_indicator,
                };
                guard.push_back(init);
            }
        }

        self.update_border_classes(ws_config.border_show.get());
    }

    pub(super) fn update_border_classes(&self, show_border: bool) {
        let container = self.buttons.widget();
        use wayle_config::schemas::bar::BorderLocation;

        for location in [
            BorderLocation::Top,
            BorderLocation::Bottom,
            BorderLocation::Left,
            BorderLocation::Right,
            BorderLocation::All,
        ] {
            if let Some(class) = location.css_class() {
                container.remove_css_class(class);
            }
        }

        if show_border && let Some(class) = self.settings.border_location.get().css_class() {
            container.add_css_class(class);
        }
    }

    fn switch_to_tag(&self, index: u32) {
        let mango = self.mango.clone();
        tokio::spawn(async move {
            if let Err(err) = mango.dispatch(&format!("view,{index}")).await {
                tracing::warn!(error = %err, "mango switch_to_tag failed");
            }
        });
    }

    fn navigate_tag(&self, direction: i32) {
        let monitor = self.settings.monitor_name.clone().unwrap_or_default();
        let mango = self.mango.clone();
        let direction = direction;

        tokio::spawn(async move {
            let Ok(response) = mango.get_tags(&monitor).await else {
                return;
            };

            let active_indices: Vec<u32> = response.tags.iter()
                .filter(|t| t.is_active)
                .map(|t| t.index)
                .collect();

            let current = active_indices.first().copied().unwrap_or(1);
            let all_indices: Vec<u32> = response.tags.iter().map(|t| t.index).collect();

            let next = if direction > 0 {
                all_indices.iter().find(|&&i| i > current).copied()
                    .or(all_indices.first().copied())
            } else {
                all_indices.iter().rev().find(|&&i| i < current).copied()
                    .or(all_indices.last().copied())
            };

            if let Some(next_index) = next {
                if let Err(err) = mango.dispatch(&format!("view,{next_index}")).await {
                    tracing::warn!(error = %err, "mango navigate_tag failed");
                }
            }
        });
    }
}
