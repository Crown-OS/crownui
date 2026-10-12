use std::{
    any::{Any, TypeId},
    cmp::Reverse,
    collections::BinaryHeap,
    panic::{AssertUnwindSafe, catch_unwind},
};

use slotmap::{Key, KeyData, SlotMap};
use smallvec::SmallVec;

use crate::{
    EffectId, FlushError, Memo, ObserverPanic, ScopeId, Signal, Store,
    handles::{ObserverId, SignalId},
    scope::{Cleanup, ScopeSlot},
};

const MAX_FLUSH_PASSES: usize = 64;

type Compute = Box<dyn FnMut(&mut Runtime) -> bool>;

struct SignalSlot {
    value: Box<dyn Any>,
    subscribers: SmallVec<[ObserverId; 4]>,
    height: u32,
}

struct ObserverSlot {
    compute: Option<Compute>,
    sources: SmallVec<[SignalId; 4]>,
    output: Option<SignalId>,
    height: u32,
    scope: ScopeId,
    queued: bool,
    quarantined: bool,
}

/// What a flush did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlushStats {
    /// Observers run.
    pub observers_run: usize,
    /// Passes needed, counting writes made by effects as new passes.
    pub passes: usize,
}

/// Owns every reactive value, observer and scope.
pub struct Runtime {
    signals: SlotMap<SignalId, SignalSlot>,
    observers: SlotMap<ObserverId, ObserverSlot>,
    scopes: SlotMap<ScopeId, ScopeSlot>,
    root: ScopeId,
    current_scope: ScopeId,
    tracking: Option<SmallVec<[SignalId; 4]>>,
    running_observer: Option<ObserverId>,
    dirty: BinaryHeap<Reverse<(u32, u64)>>,
    next_pass: Vec<ObserverId>,
    flushing: bool,
    panics: Vec<ObserverPanic>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

fn observer_key(id: ObserverId) -> u64 {
    id.data().as_ffi()
}

fn observer_from_key(key: u64) -> ObserverId {
    ObserverId::from(KeyData::from_ffi(key))
}

impl Runtime {
    /// Creates a runtime with an empty root scope.
    pub fn new() -> Self {
        let mut scopes = SlotMap::with_key();
        let root = scopes.insert(ScopeSlot::default());
        Self {
            signals: SlotMap::with_key(),
            observers: SlotMap::with_key(),
            scopes,
            root,
            current_scope: root,
            tracking: None,
            running_observer: None,
            dirty: BinaryHeap::new(),
            next_pass: Vec::new(),
            flushing: false,
            panics: Vec::new(),
        }
    }

    /// The scope that lives as long as the runtime.
    pub fn root_scope(&self) -> ScopeId {
        self.root
    }

    /// The scope new nodes are created in.
    pub fn current_scope(&self) -> ScopeId {
        self.current_scope
    }

    /// Creates a child of the current scope.
    pub fn create_scope(&mut self) -> ScopeId {
        let parent = self.current_scope;
        self.create_child_scope(parent)
    }

    /// Creates a child of `parent`.
    pub fn create_child_scope(&mut self, parent: ScopeId) -> ScopeId {
        let child = self.scopes.insert(ScopeSlot {
            parent: Some(parent),
            ..ScopeSlot::default()
        });
        if let Some(parent) = self.scopes.get_mut(parent) {
            parent.children.push(child);
        }
        child
    }

    /// Makes `scope` current and returns the previous one; pair with another call to restore it.
    pub fn set_current_scope(&mut self, scope: ScopeId) -> ScopeId {
        std::mem::replace(&mut self.current_scope, scope)
    }

    /// Runs `build` with `scope` as the current scope.
    pub fn with_scope<R>(&mut self, scope: ScopeId, build: impl FnOnce(&mut Self) -> R) -> R {
        let previous = std::mem::replace(&mut self.current_scope, scope);
        let result = build(self);
        self.current_scope = previous;
        result
    }

    /// Whether `scope` has not been disposed.
    pub fn is_scope_alive(&self, scope: ScopeId) -> bool {
        self.scopes.contains_key(scope)
    }

    /// The parent of `scope`, if any.
    pub fn parent_scope(&self, scope: ScopeId) -> Option<ScopeId> {
        self.scopes.get(scope).and_then(|slot| slot.parent)
    }

    /// Frees `scope`, its children and everything created in them, running cleanups.
    pub fn dispose_scope(&mut self, scope: ScopeId) {
        if scope == self.root {
            return;
        }
        let Some(slot) = self.scopes.remove(scope) else {
            return;
        };
        if let Some(parent) = slot.parent.and_then(|parent| self.scopes.get_mut(parent)) {
            parent.children.retain(|child| *child != scope);
        }
        for child in slot.children {
            self.dispose_scope(child);
        }
        for cleanup in slot.cleanups.into_iter().rev() {
            cleanup(self);
        }
        for observer in slot.observers {
            self.remove_observer(observer);
        }
        for signal in slot.signals {
            self.signals.remove(signal);
        }
    }

    /// Runs `cleanup` when the current scope is disposed.
    pub fn on_cleanup(&mut self, cleanup: impl FnOnce(&mut Self) + 'static) {
        let boxed: Cleanup = Box::new(cleanup);
        if let Some(scope) = self.scopes.get_mut(self.current_scope) {
            scope.cleanups.push(boxed);
        }
    }

    /// Creates a signal in the current scope.
    pub fn signal<T: 'static>(&mut self, value: T) -> Signal<T> {
        Signal::from_id(self.insert_value(Box::new(value), 0))
    }

    /// Creates untracked storage in the current scope.
    pub fn store<T: 'static>(&mut self, value: T) -> Store<T> {
        Store::from_id(self.insert_value(Box::new(value), 0))
    }

    /// Creates a memo and computes it once, now.
    pub fn memo<T: PartialEq + 'static>(
        &mut self,
        mut compute: impl FnMut(&mut Self) -> T + 'static,
    ) -> Memo<T> {
        let output = self.insert_value(Box::new(()), 0);
        let wrapped: Compute = Box::new(move |runtime: &mut Runtime| {
            let value = compute(runtime);
            let Some(slot) = runtime.signals.get_mut(output) else {
                return false;
            };
            match slot.value.downcast_mut::<T>() {
                Some(current) if *current == value => false,
                Some(current) => {
                    *current = value;
                    true
                }
                None => {
                    slot.value = Box::new(value);
                    true
                }
            }
        });
        let observer = self.insert_observer(wrapped, Some(output));
        self.run_observer(observer);
        Memo::from_id(output)
    }

    /// Creates an effect and runs it once, now.
    pub fn effect(&mut self, mut run: impl FnMut(&mut Self) + 'static) -> EffectId {
        let observer = self.insert_observer(
            Box::new(move |runtime: &mut Runtime| {
                run(runtime);
                false
            }),
            None,
        );
        self.run_observer(observer);
        EffectId(observer)
    }

    /// Stops an effect before its scope ends.
    pub fn dispose_effect(&mut self, effect: EffectId) {
        self.remove_observer(effect.0);
    }

    /// Runs `read` without subscribing the running observer to anything it reads.
    pub fn untrack<R>(&mut self, read: impl FnOnce(&mut Self) -> R) -> R {
        let saved = self.tracking.take();
        let result = read(self);
        self.tracking = saved;
        result
    }

    /// Lends two signals at once.
    pub fn with2<A: 'static, B: 'static, R>(
        &mut self,
        first: Signal<A>,
        second: Signal<B>,
        read: impl FnOnce(&A, &B) -> R,
    ) -> R {
        self.track(first.id);
        self.track(second.id);
        match (self.value::<A>(first.id), self.value::<B>(second.id)) {
            (Some(first), Some(second)) => read(first, second),
            _ => panic!("with2 on a disposed signal"),
        }
    }

    /// Provides `value` to the current scope and its descendants.
    pub fn provide<T: 'static>(&mut self, value: T) -> Signal<T> {
        let signal = self.signal(value);
        if let Some(scope) = self.scopes.get_mut(self.current_scope) {
            scope.environment.push((TypeId::of::<T>(), signal.id));
        }
        signal
    }

    /// The nearest value of type `T` provided by the current scope or an ancestor.
    pub fn environment<T: 'static>(&self) -> Option<Signal<T>> {
        let wanted = TypeId::of::<T>();
        let mut scope = Some(self.current_scope);
        while let Some(id) = scope {
            let slot = self.scopes.get(id)?;
            if let Some((_, signal)) = slot.environment.iter().find(|(kind, _)| *kind == wanted) {
                return Some(Signal::from_id(*signal));
            }
            scope = slot.parent;
        }
        None
    }

    /// Makes `scope` catch panics from observers inside it.
    pub fn set_error_boundary(
        &mut self,
        scope: ScopeId,
        handler: impl FnMut(&mut Self, &ObserverPanic) + 'static,
    ) {
        if let Some(slot) = self.scopes.get_mut(scope) {
            slot.error_handler = Some(Box::new(handler));
        }
    }

    /// Panics no error boundary handled since the last call.
    pub fn take_panics(&mut self) -> Vec<ObserverPanic> {
        std::mem::take(&mut self.panics)
    }

    /// Whether any observer waits to run.
    pub fn has_pending(&self) -> bool {
        !self.dirty.is_empty() || !self.next_pass.is_empty()
    }

    /// Number of live signals, memos and stores.
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Number of live effects and memos.
    pub fn observer_count(&self) -> usize {
        self.observers.len()
    }

    /// Number of live scopes, including the root.
    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    /// Runs dirty observers until everything is consistent.
    pub fn flush(&mut self) -> Result<FlushStats, FlushError> {
        if self.flushing {
            return Ok(FlushStats::default());
        }
        self.flushing = true;
        let mut stats = FlushStats::default();
        let result = loop {
            let pending = std::mem::take(&mut self.next_pass);
            for observer in pending {
                self.enqueue(observer);
            }
            if self.dirty.is_empty() {
                break Ok(stats);
            }
            stats.passes += 1;
            if stats.passes > MAX_FLUSH_PASSES {
                self.dirty.clear();
                self.next_pass.clear();
                for observer in self.observers.values_mut() {
                    observer.queued = false;
                }
                break Err(FlushError::Cycle {
                    passes: MAX_FLUSH_PASSES,
                });
            }
            while let Some(Reverse((_, key))) = self.dirty.pop() {
                let observer = observer_from_key(key);
                let Some(slot) = self.observers.get_mut(observer) else {
                    continue;
                };
                slot.queued = false;
                stats.observers_run += 1;
                if self.run_observer(observer)
                    && let Some(output) = self.observers.get(observer).and_then(|slot| slot.output)
                {
                    self.notify(output, true);
                }
            }
        };
        self.flushing = false;
        result
    }

    pub(crate) fn track(&mut self, signal: SignalId) {
        if let Some(sources) = self.tracking.as_mut()
            && !sources.contains(&signal)
        {
            sources.push(signal);
        }
    }

    pub(crate) fn value<T: 'static>(&self, signal: SignalId) -> Option<&T> {
        self.signals.get(signal)?.value.downcast_ref::<T>()
    }

    pub(crate) fn value_mut<T: 'static>(&mut self, signal: SignalId) -> Option<&mut T> {
        self.signals.get_mut(signal)?.value.downcast_mut::<T>()
    }

    pub(crate) fn update_value<T: 'static>(
        &mut self,
        signal: SignalId,
        change: impl FnOnce(&mut T),
    ) -> bool {
        let Some(value) = self.value_mut::<T>(signal) else {
            return false;
        };
        change(value);
        self.notify(signal, false);
        true
    }

    pub(crate) fn update_from<T: 'static, S: 'static>(
        &mut self,
        target: SignalId,
        source: SignalId,
        change: impl FnOnce(&mut T, &S),
    ) -> bool {
        if target == source {
            return false;
        }
        let Some([target_slot, source_slot]) = self.signals.get_disjoint_mut([target, source]) else {
            return false;
        };
        let (Some(target_value), Some(source_value)) = (
            target_slot.value.downcast_mut::<T>(),
            source_slot.value.downcast_ref::<S>(),
        ) else {
            return false;
        };
        change(target_value, source_value);
        self.notify(target, false);
        true
    }

    pub(crate) fn dead_handle_message(&self, signal: SignalId) -> String {
        format!(
            "CUI0001: reactive handle {signal:?} was used after its scope was disposed; \
             use try_get/try_set for values that may outlive their component"
        )
    }

    fn insert_value(&mut self, value: Box<dyn Any>, height: u32) -> SignalId {
        let scope = self.current_scope;
        let id = self.signals.insert(SignalSlot {
            value,
            subscribers: SmallVec::new(),
            height,
        });
        if let Some(slot) = self.scopes.get_mut(scope) {
            slot.signals.push(id);
        }
        id
    }

    fn insert_observer(&mut self, compute: Compute, output: Option<SignalId>) -> ObserverId {
        let scope = self.current_scope;
        let id = self.observers.insert(ObserverSlot {
            compute: Some(compute),
            sources: SmallVec::new(),
            output,
            height: 1,
            scope,
            queued: false,
            quarantined: false,
        });
        if let Some(slot) = self.scopes.get_mut(scope) {
            slot.observers.push(id);
        }
        id
    }

    fn enqueue(&mut self, observer: ObserverId) {
        let Some(slot) = self.observers.get_mut(observer) else {
            return;
        };
        if slot.queued || slot.quarantined {
            return;
        }
        slot.queued = true;
        self.dirty.push(Reverse((slot.height, observer_key(observer))));
    }

    fn notify(&mut self, signal: SignalId, from_memo: bool) {
        let Some(slot) = self.signals.get(signal) else {
            return;
        };
        let subscribers = slot.subscribers.clone();
        let from_running_effect = self.running_observer.is_some() && !from_memo;
        for observer in subscribers {
            if from_running_effect {
                if !self.next_pass.contains(&observer) {
                    self.next_pass.push(observer);
                }
            } else {
                self.enqueue(observer);
            }
        }
    }

    /// Runs one observer with tracking; returns whether its output changed.
    fn run_observer(&mut self, observer: ObserverId) -> bool {
        let Some(slot) = self.observers.get_mut(observer) else {
            return false;
        };
        if slot.quarantined {
            return false;
        }
        let Some(mut compute) = slot.compute.take() else {
            return false;
        };
        let scope = slot.scope;
        let saved_tracking = self.tracking.replace(SmallVec::new());
        let saved_observer = self.running_observer.replace(observer);
        let saved_scope = std::mem::replace(&mut self.current_scope, scope);

        let outcome = catch_unwind(AssertUnwindSafe(|| compute(self)));

        self.current_scope = saved_scope;
        self.running_observer = saved_observer;
        let sources = std::mem::replace(&mut self.tracking, saved_tracking).unwrap_or_default();

        let Some(slot) = self.observers.get_mut(observer) else {
            return false;
        };
        slot.compute = Some(compute);
        match outcome {
            Ok(changed) => {
                self.resubscribe(observer, sources);
                changed
            }
            Err(payload) => {
                slot.quarantined = true;
                let message = payload
                    .downcast_ref::<&str>()
                    .map(|text| (*text).to_owned())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "non-string panic payload".to_owned());
                self.report_panic(ObserverPanic { scope, message });
                false
            }
        }
    }

    fn resubscribe(&mut self, observer: ObserverId, sources: SmallVec<[SignalId; 4]>) {
        let previous = self
            .observers
            .get_mut(observer)
            .map(|slot| std::mem::take(&mut slot.sources))
            .unwrap_or_default();
        for old in previous.iter().filter(|old| !sources.contains(old)) {
            if let Some(slot) = self.signals.get_mut(*old) {
                slot.subscribers.retain(|subscriber| *subscriber != observer);
            }
        }
        let mut height = 1;
        for source in &sources {
            if let Some(slot) = self.signals.get_mut(*source) {
                if !slot.subscribers.contains(&observer) {
                    slot.subscribers.push(observer);
                }
                height = height.max(slot.height + 1);
            }
        }
        let Some(slot) = self.observers.get_mut(observer) else {
            return;
        };
        slot.sources = sources;
        slot.height = height;
        if let Some(output) = slot.output
            && let Some(output_slot) = self.signals.get_mut(output)
        {
            output_slot.height = height;
        }
    }

    fn remove_observer(&mut self, observer: ObserverId) {
        let Some(slot) = self.observers.remove(observer) else {
            return;
        };
        for source in slot.sources {
            if let Some(signal) = self.signals.get_mut(source) {
                signal.subscribers.retain(|subscriber| *subscriber != observer);
            }
        }
        if let Some(scope) = self.scopes.get_mut(slot.scope) {
            scope.observers.retain(|id| *id != observer);
        }
    }

    fn report_panic(&mut self, panic: ObserverPanic) {
        let mut scope = Some(panic.scope);
        while let Some(id) = scope {
            let Some(slot) = self.scopes.get_mut(id) else {
                break;
            };
            if let Some(mut handler) = slot.error_handler.take() {
                handler(self, &panic);
                if let Some(slot) = self.scopes.get_mut(id) {
                    slot.error_handler.get_or_insert(handler);
                }
                return;
            }
            scope = slot.parent;
        }
        self.panics.push(panic);
    }
}
