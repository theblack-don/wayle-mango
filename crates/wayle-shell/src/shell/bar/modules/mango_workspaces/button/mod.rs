//! Per-tag button used by the [`MangoWorkspaces`] factory.

mod methods;

use gtk::prelude::*;
use relm4::{factory::FactoryComponent, prelude::*};
use wayle_config::schemas::modules::{MangoActiveIndicator, MangoDisplayMode};

use self::methods::compute_css_classes;

#[derive(Debug, Clone)]
pub(crate) struct MangoTagButtonInit {
    pub index: u32,
    pub label: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub is_urgent: bool,
    pub has_clients: bool,
    pub is_vertical: bool,
    pub display_mode: MangoDisplayMode,
    pub active_indicator: MangoActiveIndicator,
}

pub(crate) struct MangoTagButton {
    pub(super) index: u32,
    pub(super) label: Option<String>,
    pub(super) icon: Option<String>,
    pub(super) is_vertical: bool,
    pub(super) display_mode: MangoDisplayMode,
    pub(super) classes: Vec<String>,
}

#[derive(Debug)]
pub(crate) enum MangoTagButtonInput {}

#[derive(Debug)]
pub(crate) enum MangoTagButtonOutput {
    Clicked(u32),
    ScrollUp,
    ScrollDown,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for MangoTagButton {
    type Init = MangoTagButtonInit;
    type Input = MangoTagButtonInput;
    type Output = MangoTagButtonOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            set_css_classes: &self.classes.iter().map(String::as_str).collect::<Vec<_>>(),

            connect_clicked[sender, index = self.index] => move |_| {
                let _ = sender.output(MangoTagButtonOutput::Clicked(index));
            },

            gtk::Box {
                add_css_class: "workspace-content",
                #[watch]
                set_orientation: self.orientation(),
                #[watch]
                set_halign: self.content_halign(),
                #[watch]
                set_valign: self.content_valign(),

                gtk::Label {
                    add_css_class: "workspace-label",
                    #[watch]
                    set_visible: self.show_label(),
                    #[watch]
                    set_label: self.label_text(),
                    set_valign: gtk::Align::Center,
                },

                gtk::Image {
                    add_css_class: "workspace-icon",
                    #[watch]
                    set_visible: self.show_icon(),
                    #[watch]
                    set_icon_name: self.icon.as_deref(),
                    set_valign: gtk::Align::Center,
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let classes = compute_css_classes(&init);
        Self {
            index: init.index,
            label: init.label,
            icon: init.icon,
            is_vertical: init.is_vertical,
            display_mode: init.display_mode,
            classes,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        attach_scroll(&root, &sender);

        widgets
    }

    fn update(&mut self, _msg: Self::Input, _sender: FactorySender<Self>) {}
}

fn attach_scroll(button: &gtk::Button, sender: &FactorySender<MangoTagButton>) {
    let controller = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::DISCRETE,
    );
    let sender = sender.clone();
    controller.connect_scroll(move |_, _dx, dy| {
        if dy > 0.0 {
            let _ = sender.output(MangoTagButtonOutput::ScrollDown);
        } else if dy < 0.0 {
            let _ = sender.output(MangoTagButtonOutput::ScrollUp);
        }
        gtk::glib::Propagation::Stop
    });
    button.add_controller(controller);
}
