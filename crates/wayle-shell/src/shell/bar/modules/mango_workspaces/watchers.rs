//! Background watchers: mango tag stream + config-property changes.

use std::sync::Arc;

use futures::StreamExt;
use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{
    ConfigProperty, SubscribeChanges,
    schemas::{
        modules::MangoWorkspacesConfig,
        styling::{ScaleFactor, ThemeProvider},
    },
};
use wayle_widgets::prelude::BarSettings;

use crate::services::MangoService;

use super::{MangoWorkspaces, messages::MangoWorkspacesCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<MangoWorkspaces>,
    config: &MangoWorkspacesConfig,
    mango: Arc<MangoService>,
    theme_provider: ConfigProperty<ThemeProvider>,
    bar_scale: ConfigProperty<ScaleFactor>,
    settings: &BarSettings,
) {
    let monitor = settings.monitor_name.clone();
    spawn_tag_events(sender, mango, monitor);
    spawn_config_watcher(sender, config, theme_provider, bar_scale, settings);
}

fn spawn_tag_events(
    sender: &ComponentSender<MangoWorkspaces>,
    mango: Arc<MangoService>,
    monitor: Option<String>,
) {
    sender.command(move |out, shutdown| watch_tag_events(mango, monitor, out, shutdown));
}

async fn watch_tag_events(
    mango: Arc<MangoService>,
    monitor: Option<String>,
    out: relm4::Sender<MangoWorkspacesCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let monitor_name = monitor.unwrap_or_default();
    let mut events = mango.watch_tags(monitor_name);
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            event = events.next() => {
                let Some(result) = event else { return };
                if result.is_err() {
                    continue;
                }
                let _ = out.send(MangoWorkspacesCmd::TagsChanged);
            }
        }
    }
}

fn spawn_config_watcher(
    sender: &ComponentSender<MangoWorkspaces>,
    config: &MangoWorkspacesConfig,
    theme_provider: ConfigProperty<ThemeProvider>,
    bar_scale: ConfigProperty<ScaleFactor>,
    settings: &BarSettings,
) {
    let (tx, rx) = mpsc::unbounded_channel();

    config.subscribe_changes(tx.clone());
    theme_provider.subscribe_changes(tx.clone());
    bar_scale.subscribe_changes(tx.clone());
    settings.border_width.subscribe_changes(tx.clone());
    settings.border_location.subscribe_changes(tx.clone());
    settings.is_vertical.subscribe_changes(tx);

    sender.command(move |out, shutdown| watch_config_changes(rx, out, shutdown));
}

async fn watch_config_changes(
    mut rx: mpsc::UnboundedReceiver<()>,
    out: relm4::Sender<MangoWorkspacesCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            received = rx.recv() => {
                if received.is_none() {
                    return;
                }

                while rx.try_recv().is_ok() {}

                let _ = out.send(MangoWorkspacesCmd::ConfigChanged);
            }
        }
    }
}
