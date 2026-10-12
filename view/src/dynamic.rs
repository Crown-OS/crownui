use std::{collections::HashMap, hash::Hash};

use reactive::{ScopeId, Signal};
use semantics::Role;

use crate::{
    Cx, NodeId, View,
    cx::Command,
    element::{Element, Kind},
    prop::{IntoProp, Prop},
};

pub(crate) type Rebuild = Box<dyn FnMut(&mut Cx, NodeId)>;

/// Children that are rebuilt when signals change.
pub struct DynamicChildren {
    rebuild: Rebuild,
}

impl Cx {
    pub(crate) fn install_rebuild(&mut self, node: NodeId, rebuild: Rebuild) {
        self.install_dynamic(node, rebuild);
        self.rebuild_dynamic(node);
    }

    pub(crate) fn mount_row<T: 'static, V: View>(
        &mut self,
        item: T,
        row: &mut impl FnMut(&mut Cx, Signal<T>) -> V,
    ) -> (ScopeId, NodeId, Signal<T>) {
        let scope = self.create_scope();
        let previous = self.set_current_scope(scope);
        let item_signal = self.signal(item);
        let root = row(self, item_signal).build(self);
        self.set_current_scope(previous);
        (scope, root, item_signal)
    }

    pub(crate) fn unmount_row(&mut self, scope: ScopeId, root: NodeId) {
        self.unmount_branch(scope, root);
    }

    fn install_dynamic(&mut self, node: NodeId, rebuild: Rebuild) {
        if let Some(element) = self.tree.get_mut(node) {
            element.dynamic = Some(DynamicChildren { rebuild });
        }
    }

    pub(crate) fn rebuild_dynamic(&mut self, node: NodeId) {
        let Some(mut dynamic) = self.tree.get_mut(node).and_then(|element| element.dynamic.take()) else {
            return;
        };
        let scope = self.scope_of(node);
        let previous = scope.map(|scope| self.set_current_scope(scope));
        (dynamic.rebuild)(self, node);
        if let Some(previous) = previous {
            self.set_current_scope(previous);
        }
        if let Some(element) = self.tree.get_mut(node) {
            element.dynamic = Some(dynamic);
        }
    }

    fn mount_branch(&mut self, build: impl FnOnce(&mut Cx) -> NodeId) -> (ScopeId, NodeId) {
        let scope = self.create_scope();
        let previous = self.set_current_scope(scope);
        let root = build(self);
        self.set_current_scope(previous);
        (scope, root)
    }

    fn unmount_branch(&mut self, scope: ScopeId, root: NodeId) {
        self.tree.remove_subtree(root);
        self.dispose_scope(scope);
    }

    /// Collapses `wrapper` out of layout exactly while its only child is collapsed, so a branch
    /// that builds nothing leaves no gap between its siblings.
    fn follow_collapse(&mut self, wrapper: NodeId, child: NodeId) {
        let display_of = |cx: &Self, node: NodeId| cx.tree.get(node).map(|element| element.style.layout.display);
        let collapsed = display_of(self, child) == Some(layouting::Display::None);
        let wrapper_collapsed = display_of(self, wrapper) == Some(layouting::Display::None);
        if collapsed != wrapper_collapsed {
            let display = if collapsed {
                layouting::Display::None
            } else {
                layouting::Display::Flex
            };
            self.apply_style(wrapper, &crate::StyleProp::Display(display));
        }
    }

    pub(crate) fn queue_rebuild_when(&mut self, node: NodeId, mut watch: impl FnMut(&mut reactive::Runtime) + 'static) {
        let commands = self.commands;
        let mut primed = false;
        self.effect(move |runtime| {
            watch(runtime);
            if std::mem::replace(&mut primed, true) {
                commands.update(runtime, |queue| queue.push(Command::Rebuild(node)));
            }
        });
    }
}

/// The kind behind [`show`].
pub struct ShowKind<T, E> {
    condition: Prop<bool>,
    then: T,
    otherwise: E,
}

impl<T, E, A, B> Kind for ShowKind<T, E>
where
    T: FnMut() -> A + 'static,
    E: FnMut() -> B + 'static,
    A: View,
    B: View,
{
    const NAME: &'static str = "show";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            condition,
            mut then,
            mut otherwise,
        } = self;
        let state = cx.store(true);
        match condition {
            Prop::Static(value) => state.update(cx, |current| *current = value),
            Prop::Dynamic(mut compute) => {
                let initial = cx.untrack(|runtime| compute(runtime));
                state.update(cx, |current| *current = initial);
                cx.queue_rebuild_when(node, move |runtime| {
                    let value = compute(runtime);
                    state.update(runtime, |current| *current = value);
                });
            }
        }
        let mut mounted: Option<(bool, ScopeId, NodeId)> = None;
        let rebuild: Rebuild = Box::new(move |cx, node| {
            let wanted = state.with(cx, |value| *value);
            if mounted.is_some_and(|(branch, _, _)| branch == wanted) {
                return;
            }
            if let Some((_, scope, root)) = mounted.take() {
                cx.unmount_branch(scope, root);
            }
            let (scope, root) = if wanted {
                cx.mount_branch(|cx| then().build(cx))
            } else {
                cx.mount_branch(|cx| otherwise().build(cx))
            };
            cx.tree.set_children(node, vec![root]);
            cx.follow_collapse(node, root);
            mounted = Some((wanted, scope, root));
        });
        cx.install_dynamic(node, rebuild);
        cx.rebuild_dynamic(node);
    }
}

/// Shows `then()` while `condition` holds and `otherwise()` when it does not. The hidden branch
/// is unmounted, freeing its signals, effects and tasks. A branch styled `display_none` takes no
/// space, and no gap, among the `show`'s siblings.
pub fn show<A, B, T, E>(condition: impl IntoProp<bool>, then: T, otherwise: E) -> Element<ShowKind<T, E>>
where
    A: View,
    B: View,
    T: FnMut() -> A + 'static,
    E: FnMut() -> B + 'static,
{
    Element::new(ShowKind {
        condition: condition.into_prop(),
        then,
        otherwise,
    })
}

/// The kind behind [`keyed`].
pub struct KeyedKind<C, B> {
    key: C,
    build: B,
}

impl<T, C, B, V> Kind for KeyedKind<C, B>
where
    T: Clone + PartialEq + 'static,
    C: FnMut(&mut reactive::Runtime) -> T + 'static,
    B: FnMut(&mut Cx, T) -> V + 'static,
    V: View,
{
    const NAME: &'static str = "keyed";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self { key, mut build } = self;
        cx.set_default_style(
            node,
            crate::StyleProp::FlexDirection(layouting::FlexDirection::Column),
        );
        let current = cx.memo(key);
        cx.queue_rebuild_when(node, move |runtime| current.with(runtime, |_| ()));
        let mut mounted: Option<(ScopeId, NodeId)> = None;
        let rebuild: Rebuild = Box::new(move |cx, node| {
            if let Some((scope, root)) = mounted.take() {
                cx.unmount_branch(scope, root);
            }
            let value = cx.untrack(|runtime| current.get(runtime));
            let (scope, root) = cx.mount_branch(|cx| build(cx, value).build(cx));
            cx.tree.set_children(node, vec![root]);
            cx.follow_collapse(node, root);
            mounted = Some((scope, root));
        });
        cx.install_dynamic(node, rebuild);
        cx.rebuild_dynamic(node);
    }
}

/// Builds `build(cx, value)` from the value `key` derives, and rebuilds it only when that value
/// changes: the generalisation of [`show`] to any number of branches. Everything inside reads
/// finer-grained state through bindings, so `key` should name only what changes the structure.
/// A branch styled `display_none` takes no space, and no gap, among the `keyed`'s siblings.
pub fn keyed<T, V, C, B>(key: C, build: B) -> Element<KeyedKind<C, B>>
where
    T: Clone + PartialEq + 'static,
    C: FnMut(&mut reactive::Runtime) -> T + 'static,
    B: FnMut(&mut Cx, T) -> V + 'static,
    V: View,
{
    Element::new(KeyedKind { key, build })
}

/// The kind behind [`for_each`].
pub struct ForEachKind<T, K, F> {
    items: Signal<Vec<T>>,
    key: fn(&T) -> K,
    row: F,
}

struct Row<T> {
    scope: ScopeId,
    root: NodeId,
    item: Signal<T>,
}

impl<T, K, F, V> Kind for ForEachKind<T, K, F>
where
    T: Clone + PartialEq + 'static,
    K: Hash + Eq + Clone + 'static,
    F: FnMut(&mut Cx, Signal<T>) -> V + 'static,
    V: View,
{
    const NAME: &'static str = "for_each";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self { items, key, mut row } = self;
        cx.queue_rebuild_when(node, move |runtime| items.with(runtime, |_| ()));
        let mut rows: HashMap<K, Row<T>> = HashMap::new();
        let rebuild: Rebuild = Box::new(move |cx, node| {
            let current: Vec<T> = items.with_untracked(cx, Clone::clone);
            let mut order = Vec::with_capacity(current.len());
            let mut next_rows = HashMap::with_capacity(current.len());
            for item in current {
                let item_key = key(&item);
                let entry = match rows.remove(&item_key) {
                    Some(existing) => {
                        if existing.item.with_untracked(cx, |old| *old != item) {
                            existing.item.set(cx, item);
                        }
                        existing
                    }
                    None => {
                        let (scope, root, item_signal) = {
                            let scope = cx.create_scope();
                            let previous = cx.set_current_scope(scope);
                            let item_signal = cx.signal(item);
                            let root = row(cx, item_signal).build(cx);
                            cx.set_current_scope(previous);
                            (scope, root, item_signal)
                        };
                        if let Some(element) = cx.tree.get_mut(root) {
                            use std::hash::Hasher;
                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                            item_key.hash(&mut hasher);
                            element.key = Some(hasher.finish());
                        }
                        Row {
                            scope,
                            root,
                            item: item_signal,
                        }
                    }
                };
                order.push(entry.root);
                next_rows.insert(item_key, entry);
            }
            for (_, stale) in rows.drain() {
                cx.unmount_branch(stale.scope, stale.root);
            }
            rows = next_rows;
            cx.tree.set_children(node, order);
        });
        cx.set_role(node, Role::List);
        cx.install_dynamic(node, rebuild);
        cx.rebuild_dynamic(node);
    }
}

/// One child per item, keyed so rows keep their state across inserts, removals and moves.
/// Each row receives its own `Signal<T>`; editing one item only touches that row.
pub fn for_each<T, K, V, F>(items: Signal<Vec<T>>, key: fn(&T) -> K, row: F) -> Element<ForEachKind<T, K, F>>
where
    T: Clone + PartialEq + 'static,
    K: Hash + Eq + Clone + 'static,
    V: View,
    F: FnMut(&mut Cx, Signal<T>) -> V + 'static,
{
    Element::new(ForEachKind { items, key, row })
}

/// The kind behind [`error_boundary`].
pub struct ErrorBoundaryKind<C, F> {
    child: C,
    fallback: F,
}

impl<C, F, A, B> Kind for ErrorBoundaryKind<C, F>
where
    C: FnOnce() -> A + 'static,
    F: FnMut(String) -> B + 'static,
    A: View,
    B: View,
{
    const NAME: &'static str = "error_boundary";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self { child, mut fallback } = self;
        let (scope, root) = cx.mount_branch(|cx| child().build(cx));
        let failure = cx.store(None::<String>);
        let commands = cx.commands;
        cx.set_error_boundary(scope, move |runtime, panic| {
            let message = panic.message.clone();
            failure.update(runtime, |slot| *slot = Some(message));
            commands.update(runtime, |queue| queue.push(Command::Rebuild(node)));
        });
        let mut mounted = Some((scope, root));
        let rebuild: Rebuild = Box::new(move |cx, node| {
            let Some(message) = failure.update(cx, Option::take) else {
                return;
            };
            tracing::error!(path = %cx.tree.path(node), "CUI0301: component panicked: {message}");
            if let Some((scope, root)) = mounted.take() {
                cx.unmount_branch(scope, root);
            }
            let (scope, root) = cx.mount_branch(|cx| fallback(message).build(cx));
            cx.tree.set_children(node, vec![root]);
            mounted = Some((scope, root));
        });
        cx.tree.set_children(node, vec![root]);
        cx.install_dynamic(node, rebuild);
    }
}

/// Shows `fallback(message)` instead of `child()` when anything inside panics.
pub fn error_boundary<A, B, C, F>(child: C, fallback: F) -> Element<ErrorBoundaryKind<C, F>>
where
    A: View,
    B: View,
    C: FnOnce() -> A + 'static,
    F: FnMut(String) -> B + 'static,
{
    Element::new(ErrorBoundaryKind { child, fallback })
}
