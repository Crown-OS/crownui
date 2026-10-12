use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::{action::parse_action, query::NodeQuery};

pub(crate) const USAGE: &str = "\
usage: crowndev [--socket-dir <dir>] <command>

commands:
  apps                          list running CrownUI apps
  tree [app]                    print the semantic tree
  inspect [app]                 print the inspect snapshot as JSON
  act <app> <query> <action>    e.g. act counter 'role=button,name=+1' click
  watch [app]                   reprint the tree whenever it changes (Ctrl-C to stop)
  help                          show this message";

/// What the user asked for.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Command {
    Apps,
    Tree {
        app: Option<String>,
    },
    Inspect {
        app: Option<String>,
    },
    Act {
        app: String,
        query: NodeQuery,
        action: Value,
    },
    Watch {
        app: Option<String>,
    },
    Help,
}

/// A parsed command line.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Invocation {
    pub(crate) socket_dir: Option<PathBuf>,
    pub(crate) command: Command,
}

pub(crate) fn parse_args(arguments: impl IntoIterator<Item = String>) -> Result<Invocation> {
    let mut arguments = arguments.into_iter();
    let mut socket_dir = None;
    let mut positional = Vec::new();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--socket-dir" => {
                let directory = arguments.next().context("--socket-dir needs a directory")?;
                socket_dir = Some(PathBuf::from(directory));
            }
            "-h" | "--help" => positional = vec!["help".to_owned()],
            flag if flag.starts_with("--") => bail!("unknown option `{flag}`\n\n{USAGE}"),
            _ => positional.push(argument),
        }
    }
    Ok(Invocation {
        socket_dir,
        command: parse_command(positional)?,
    })
}

fn parse_command(positional: Vec<String>) -> Result<Command> {
    let mut words = positional.into_iter();
    let Some(name) = words.next() else {
        return Ok(Command::Help);
    };
    let rest: Vec<String> = words.collect();
    let optional_app = |rest: Vec<String>| -> Result<Option<String>> {
        match <[String; 1]>::try_from(rest) {
            Ok([app]) => Ok(Some(app)),
            Err(rest) if rest.is_empty() => Ok(None),
            Err(_) => bail!("`{name}` takes at most one app\n\n{USAGE}"),
        }
    };
    Ok(match name.as_str() {
        "apps" if rest.is_empty() => Command::Apps,
        "tree" => Command::Tree {
            app: optional_app(rest)?,
        },
        "inspect" => Command::Inspect {
            app: optional_app(rest)?,
        },
        "watch" => Command::Watch {
            app: optional_app(rest)?,
        },
        "act" => {
            let Ok([app, query, action]) = <[String; 3]>::try_from(rest) else {
                bail!("`act` needs <app> <query> <action>\n\n{USAGE}");
            };
            Command::Act {
                app,
                query: NodeQuery::parse(&query)?,
                action: parse_action(&action)?,
            }
        }
        "help" => Command::Help,
        _ => bail!("unknown command `{name}`\n\n{USAGE}"),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(line: &str) -> Result<Invocation> {
        parse_args(line.split_whitespace().map(str::to_owned))
    }

    #[test]
    fn commands_with_optional_apps() -> Result<()> {
        assert_eq!(parse("apps")?.command, Command::Apps);
        assert_eq!(parse("tree")?.command, Command::Tree { app: None });
        assert_eq!(
            parse("inspect notes")?.command,
            Command::Inspect {
                app: Some("notes".into())
            }
        );
        assert_eq!(
            parse("watch counter-42")?.command,
            Command::Watch {
                app: Some("counter-42".into())
            }
        );
        Ok(())
    }

    #[test]
    fn act_parses_its_query_and_action() -> Result<()> {
        let Command::Act { app, query, action } =
            parse("act counter role=button,name=+1 click")?.command
        else {
            panic!("expected act");
        };
        assert_eq!(app, "counter");
        assert_eq!(query.name.as_deref(), Some("+1"));
        assert_eq!(action, json!({ "action": "click" }));
        Ok(())
    }

    #[test]
    fn socket_dir_and_help_flags() -> Result<()> {
        let invocation = parse("--socket-dir /tmp/apps tree")?;
        assert_eq!(invocation.socket_dir, Some(PathBuf::from("/tmp/apps")));
        assert_eq!(parse("")?.command, Command::Help);
        assert_eq!(parse("tree --help")?.command, Command::Help);
        Ok(())
    }

    #[test]
    fn malformed_command_lines_are_rejected() {
        for line in [
            "frobnicate",
            "apps extra",
            "tree a b",
            "act counter",
            "act a role=button explode",
            "--verbose",
            "--socket-dir",
        ] {
            assert!(parse(line).is_err(), "{line}");
        }
    }
}
