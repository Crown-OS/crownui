use anyhow::{Result, bail};
use serde_json::{Map, Value};

const KEYS: [&str; 3] = ["role", "name", "text"];

/// A `ui_query` filter parsed from `role=…,name=…,text=…`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct NodeQuery {
    pub(crate) role: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) text: Option<String>,
}

impl NodeQuery {
    pub(crate) fn parse(source: &str) -> Result<Self> {
        let mut query = Self::default();
        for (key, value) in fields(source)? {
            let slot = match key {
                "role" => &mut query.role,
                "name" => &mut query.name,
                _ => &mut query.text,
            };
            if slot.replace(value.to_owned()).is_some() {
                bail!("`{key}` appears twice in query `{source}`");
            }
        }
        Ok(query)
    }

    pub(crate) fn to_arguments(&self) -> Map<String, Value> {
        KEYS.into_iter()
            .zip([&self.role, &self.name, &self.text])
            .filter_map(|(key, value)| Some((key.to_owned(), Value::from(value.clone()?))))
            .collect()
    }
}

fn starts_with_key(segment: &str) -> bool {
    segment
        .split_once('=')
        .is_some_and(|(key, _)| KEYS.contains(&key.trim()))
}

fn fields(source: &str) -> Result<Vec<(&str, &str)>> {
    let mut starts: Vec<usize> = vec![0];
    starts.extend(
        source
            .match_indices(',')
            .map(|(index, _)| index + 1)
            .filter(|&start| starts_with_key(&source[start..])),
    );
    let ends = starts
        .iter()
        .skip(1)
        .map(|start| start - 1)
        .chain([source.len()]);
    starts
        .iter()
        .zip(ends)
        .map(|(&start, end)| {
            let segment = &source[start..end];
            match segment.split_once('=') {
                Some((key, value)) if KEYS.contains(&key.trim()) => Ok((key.trim(), value)),
                _ => bail!("expected role=…, name=… or text=… in query, found `{segment}`"),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_parse_in_any_order() -> Result<()> {
        let query = NodeQuery::parse("name=+1,role=button")?;
        assert_eq!(query.role.as_deref(), Some("button"));
        assert_eq!(query.name.as_deref(), Some("+1"));
        assert_eq!(query.text, None);
        Ok(())
    }

    #[test]
    fn commas_not_followed_by_a_key_stay_in_the_value() -> Result<()> {
        let query = NodeQuery::parse("text=Hello, world,role=text")?;
        assert_eq!(query.text.as_deref(), Some("Hello, world"));
        assert_eq!(query.role.as_deref(), Some("text"));
        Ok(())
    }

    #[test]
    fn values_may_contain_equals_signs() -> Result<()> {
        assert_eq!(NodeQuery::parse("name=a=b")?.name.as_deref(), Some("a=b"));
        Ok(())
    }

    #[test]
    fn unknown_keys_duplicates_and_bare_words_are_rejected() {
        assert!(NodeQuery::parse("colour=red").is_err());
        assert!(NodeQuery::parse("role=button,role=text").is_err());
        assert!(NodeQuery::parse("button").is_err());
        assert!(NodeQuery::parse("").is_err());
    }

    #[test]
    fn arguments_carry_only_the_given_keys() -> Result<()> {
        let arguments = NodeQuery::parse("role=button")?.to_arguments();
        assert_eq!(
            Value::Object(arguments),
            serde_json::json!({ "role": "button" })
        );
        Ok(())
    }
}
