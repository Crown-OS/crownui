use std::fmt::Write;

use crate::{Role, SemanticId, SemanticNode, SemanticTree};

/// Finds nodes the way users perceive them: by role, accessible name or visible text.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Query {
    role: Option<Role>,
    name: Option<String>,
    text: Option<String>,
}

/// Starts a query for nodes with `role`.
pub fn role(role: Role) -> Query {
    Query::default().role(role)
}

/// Starts a query for nodes whose accessible name is exactly `name`.
pub fn name(name: impl Into<String>) -> Query {
    Query::default().name(name)
}

/// Starts a query for nodes whose name or value contains `text`.
pub fn text(text: impl Into<String>) -> Query {
    Query::default().text(text)
}

/// Why a query did not find exactly one node.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum QueryError {
    /// Nothing matched.
    #[error("CUI0101: no node matches {query}; similar nodes:\n{candidates}")]
    NotFound {
        /// The query, described.
        query: String,
        /// Nodes with the same role or a similar name, one per line.
        candidates: String,
    },
    /// More than one node matched.
    #[error("CUI0102: {count} nodes match {query}; narrow it down:\n{matches}")]
    Ambiguous {
        /// The query, described.
        query: String,
        /// How many matched.
        count: usize,
        /// The matching nodes, one per line.
        matches: String,
    },
}

impl Query {
    /// Requires `role`.
    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Requires the accessible name to equal `name`.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Requires the name or exposed value to contain `text`.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Whether `node` matches.
    pub fn matches(&self, node: &SemanticNode) -> bool {
        self.role.is_none_or(|role| node.role == role)
            && self
                .name
                .as_deref()
                .is_none_or(|name| node.name.as_deref() == Some(name))
            && self.text.as_deref().is_none_or(|text| {
                node.name.as_deref().is_some_and(|name| name.contains(text))
                    || node.exposed_value().is_some_and(|value| value.contains(text))
            })
    }

    /// Every matching node, in tree order.
    pub fn find_all<'tree>(&self, tree: &'tree SemanticTree) -> Vec<&'tree SemanticNode> {
        tree.iter().filter(|node| self.matches(node)).collect()
    }

    /// The single matching node.
    pub fn find_one(&self, tree: &SemanticTree) -> Result<SemanticId, QueryError> {
        let found = self.find_all(tree);
        match found.as_slice() {
            [node] => Ok(node.id),
            [] => Err(QueryError::NotFound {
                query: self.to_string(),
                candidates: describe(tree.iter().filter(|node| {
                    self.role.is_some_and(|role| node.role == role)
                        || node.name.is_some() && (self.name.is_some() || self.text.is_some())
                })),
            }),
            many => Err(QueryError::Ambiguous {
                query: self.to_string(),
                count: many.len(),
                matches: describe(many.iter().copied()),
            }),
        }
    }
}

fn describe<'tree>(nodes: impl Iterator<Item = &'tree SemanticNode>) -> String {
    let mut out = String::new();
    for node in nodes.take(10) {
        let _ = writeln!(out, "  {} {:?} at {}", node.role, node.name.as_deref().unwrap_or(""), node.path);
    }
    out
}

impl std::fmt::Display for Query {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if let Some(role) = self.role {
            parts.push(format!("role({role})"));
        }
        if let Some(name) = &self.name {
            parts.push(format!("name({name:?})"));
        }
        if let Some(text) = &self.text {
            parts.push(format!("text({text:?})"));
        }
        formatter.write_str(&parts.join("."))
    }
}
