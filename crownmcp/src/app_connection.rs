use anyhow::{Context, Result};
use mcp::AppEndpoint;
use rmcp::{Peer, RoleClient, ServiceExt, service::RunningService};

pub(crate) struct AppConnection {
    pub(crate) endpoint: AppEndpoint,
    client: RunningService<RoleClient, ()>,
}

impl AppConnection {
    pub(crate) async fn open(endpoint: AppEndpoint, token: Option<&str>) -> Result<Self> {
        let stream = mcp::connect(&endpoint.path, token)
            .await
            .with_context(|| format!("cannot reach {}", endpoint.label()))?;
        let client = ().serve(stream).await.with_context(|| {
            format!(
                "{} refused the connection; is CROWNMCP_TOKEN right?",
                endpoint.label()
            )
        })?;
        Ok(Self { endpoint, client })
    }

    pub(crate) fn peer(&self) -> &Peer<RoleClient> {
        self.client.peer()
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.client.is_closed()
    }
}
