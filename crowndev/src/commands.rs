use std::{io::Write, path::PathBuf, time::Duration};

use anyhow::{Result, bail};
use serde_json::Value;

use crate::{
    app_client::{AppClient, QueryMatch},
    cli::{Command, USAGE},
    discovery::{discover_apps, select_app},
    query::NodeQuery,
    tree_watcher::TreeWatcher,
};

const WATCH_INTERVAL: Duration = Duration::from_millis(500);

/// Where apps are found and how to authenticate to them.
pub(crate) struct Session {
    pub(crate) socket_dir: PathBuf,
    pub(crate) token: Option<String>,
}

impl Session {
    async fn with_app<T>(
        &self,
        app: Option<&str>,
        work: impl AsyncFnOnce(&AppClient) -> Result<T>,
    ) -> Result<T> {
        let endpoint = select_app(discover_apps(&self.socket_dir), app)?;
        let client = AppClient::connect(&endpoint, self.token.as_deref()).await?;
        let outcome = work(&client).await;
        client.close().await?;
        outcome
    }
}

pub(crate) async fn run(command: &Command, session: &Session, out: &mut impl Write) -> Result<()> {
    match command {
        Command::Help => writeln!(out, "{USAGE}")?,
        Command::Apps => list_apps(session, out)?,
        Command::Tree { app } => {
            let tree = session
                .with_app(app.as_deref(), async |client| client.tree_text().await)
                .await?;
            writeln!(out, "{tree}")?;
        }
        Command::Inspect { app } => {
            let snapshot = session
                .with_app(app.as_deref(), async |client| client.inspect().await)
                .await?;
            writeln!(out, "{}", serde_json::to_string_pretty(&snapshot)?)?;
        }
        Command::Act { app, query, action } => {
            let report = session
                .with_app(Some(app), async |client| act(client, query, action).await)
                .await?;
            writeln!(out, "{report}")?;
        }
        Command::Watch { app } => {
            session
                .with_app(app.as_deref(), async |client| watch(client, out).await)
                .await?
        }
    }
    Ok(())
}

fn list_apps(session: &Session, out: &mut impl Write) -> Result<()> {
    let apps = discover_apps(&session.socket_dir);
    if apps.is_empty() {
        writeln!(
            out,
            "no CrownUI app is running in {}",
            session.socket_dir.display()
        )?;
    }
    for app in apps {
        writeln!(out, "{}\t{}", app.label(), app.path.display())?;
    }
    Ok(())
}

fn describe(node: &QueryMatch) -> String {
    format!(
        "{} {:?} (id {})",
        node.role,
        node.name.as_deref().unwrap_or_default(),
        node.id
    )
}

async fn act(client: &AppClient, query: &NodeQuery, action: &Value) -> Result<String> {
    let found = client.query(query).await?;
    let target = match found.matches.as_slice() {
        [target] => target,
        [] => bail!("no node in {} matches the query", client.label()),
        several => bail!(
            "{} nodes match; narrow the query:\n  {}",
            several.len(),
            several
                .iter()
                .map(describe)
                .collect::<Vec<_>>()
                .join("\n  ")
        ),
    };
    let revision = client.act(target.id, found.revision, action).await?;
    let name = action["action"].as_str().unwrap_or("act");
    Ok(format!(
        "{name} {} -> revision {revision}",
        describe(target)
    ))
}

async fn watch(client: &AppClient, out: &mut impl Write) -> Result<()> {
    eprintln!("watching {}; Ctrl-C to stop", client.label());
    let mut watcher = TreeWatcher::default();
    loop {
        if let Some(tree) = watcher.observe(client.tree_text().await?) {
            writeln!(out, "{tree}\n---")?;
            out.flush()?;
        }
        tokio::time::sleep(WATCH_INTERVAL).await;
    }
}
