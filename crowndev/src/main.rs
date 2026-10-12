#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod action;
mod app_client;
mod cli;
mod commands;
mod discovery;
mod query;
#[cfg(test)]
mod tests;
mod tree_watcher;

use commands::Session;

const TOKEN_VARIABLE: &str = "CROWNMCP_TOKEN";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let invocation = cli::parse_args(std::env::args().skip(1))?;
    let session = Session {
        socket_dir: invocation
            .socket_dir
            .unwrap_or_else(mcp::default_socket_dir),
        token: std::env::var(TOKEN_VARIABLE).ok(),
    };
    commands::run(&invocation.command, &session, &mut std::io::stdout().lock()).await
}
