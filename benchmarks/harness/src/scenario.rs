use std::str::FromStr;

use anyhow::anyhow;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scenario {
    Hello,
    List,
    Animation,
    Settings,
}

impl Scenario {
    pub const ALL: [Self; 4] = [Self::Hello, Self::List, Self::Animation, Self::Settings];

    pub fn name(self) -> &'static str {
        match self {
            Self::Hello => "hello",
            Self::List => "list",
            Self::Animation => "animation",
            Self::Settings => "settings",
        }
    }
}

impl FromStr for Scenario {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> anyhow::Result<Self> {
        Self::ALL
            .into_iter()
            .find(|scenario| scenario.name() == name)
            .ok_or_else(|| {
                anyhow!("unknown scenario {name:?}; expected hello, list, animation or settings")
            })
    }
}
