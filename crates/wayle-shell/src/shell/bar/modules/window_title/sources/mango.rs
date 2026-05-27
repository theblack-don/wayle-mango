//! MangoWM implementation of [`FocusedWindowSource`].

use std::sync::Arc;

use futures::{StreamExt, stream::BoxStream};
use tokio::{runtime::Handle, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::services::MangoService;

use super::{FocusedWindow, FocusedWindowSource};

pub(crate) struct MangoFocusedWindowSource {
    service: Arc<MangoService>,
}

impl MangoFocusedWindowSource {
    pub(crate) fn new(service: Arc<MangoService>) -> Self {
        Self { service }
    }
}

impl FocusedWindowSource for MangoFocusedWindowSource {
    fn snapshot(&self) -> Option<FocusedWindow> {
        let runtime = Handle::current();
        let client = runtime.block_on(self.service.get_focusing_client()).ok()??;
        Some(FocusedWindow {
            title: client.title.unwrap_or_default(),
            app_id: client.app_id.unwrap_or_default(),
        })
    }

    fn changes(&self) -> BoxStream<'static, Option<FocusedWindow>> {
        let service = Arc::clone(&self.service);
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut events = service.watch_focusing_client();
            while let Some(result) = events.next().await {
                let focused = match result {
                    Ok(Some(client)) => Some(FocusedWindow {
                        title: client.title.unwrap_or_default(),
                        app_id: client.app_id.unwrap_or_default(),
                    }),
                    Ok(None) => None,
                    Err(_) => continue,
                };
                if tx.send(focused).is_err() {
                    return;
                }
            }
        });

        Box::pin(UnboundedReceiverStream::new(rx))
    }
}
