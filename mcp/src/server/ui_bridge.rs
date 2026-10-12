use std::time::Duration;

use tokio::{
    sync::{mpsc, watch},
    time::timeout,
};

use crate::{McpRequest, Responder, handle::Wakeup};

const UI_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub(crate) enum UiError {
    #[error("the app did not answer within {} s", UI_TIMEOUT.as_secs())]
    Timeout,
    #[error("the app stopped serving MCP requests")]
    Closed,
}

#[derive(Clone)]
pub(crate) struct UiBridge {
    requests: mpsc::Sender<McpRequest>,
    wakeup: watch::Receiver<Option<Wakeup>>,
}

impl UiBridge {
    pub(crate) fn new(
        requests: mpsc::Sender<McpRequest>,
        wakeup: watch::Receiver<Option<Wakeup>>,
    ) -> Self {
        Self { requests, wakeup }
    }

    pub(crate) async fn ask<T>(
        &self,
        request: impl FnOnce(Responder<T>) -> McpRequest,
    ) -> Result<T, UiError> {
        let (responder, reply) = Responder::new();
        let round_trip = async {
            self.requests
                .send(request(responder))
                .await
                .map_err(|_| UiError::Closed)?;
            let wakeup = self.wakeup.borrow().clone();
            if let Some(wakeup) = wakeup {
                wakeup();
            }
            reply.await.map_err(|_| UiError::Closed)
        };
        timeout(UI_TIMEOUT, round_trip)
            .await
            .map_err(|_| UiError::Timeout)?
    }
}
