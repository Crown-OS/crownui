use serde_json::{Value, json};

use crate::TreeSnapshot;

pub(super) fn tree_json(snapshot: &TreeSnapshot) -> Value {
    let nodes: Vec<Value> = snapshot
        .tree
        .iter()
        .filter_map(|node| {
            let mut value = serde_json::to_value(node).ok()?;
            if node.states.secure
                && let Some(object) = value.as_object_mut()
            {
                object.insert("value".into(), Value::Null);
            }
            Some(value)
        })
        .collect();
    json!({ "revision": snapshot.revision, "root": snapshot.tree.root(), "nodes": nodes })
}
