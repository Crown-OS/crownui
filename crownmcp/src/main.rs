#![doc = include_str!("../README.md")]

mod app_connection;
mod broker;
mod broker_tools;
mod discovery;
#[cfg(test)]
mod tests;

use anyhow::Context;
use rmcp::ServiceExt;

use broker::Broker;

const TOKEN_VARIABLE: &str = "CROWNMCP_TOKEN";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let broker = Broker::new(
        mcp::default_socket_dir(),
        std::env::var(TOKEN_VARIABLE).ok(),
    );
    broker
        .serve(rmcp::transport::stdio())
        .await
        .context("MCP handshake with the agent failed")?
        .waiting()
        .await?;
    Ok(())
}
