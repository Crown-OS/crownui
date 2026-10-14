use std::marker::PhantomData;

use slotmap::new_key_type;

use crate::Runtime;

new_key_type! {
    pub(crate) struct SignalId;
    pub(crate) struct ObserverId;
}

/// A reactive value. Reading it inside a memo or effect subscribes that observer.
pub struct Signal<T> {
    pub(crate) id: SignalId,
    marker: PhantomData<fn() -> T>,
}

/// A derived value recomputed when what it read changes; unchanged results stop propagation.
pub struct Memo<T> {
    pub(crate) id: SignalId,
    marker: PhantomData<fn() -> T>,
}

/// Untracked storage owned by a scope: reads subscribe nothing and writes notify nobody.
pub struct Store<T> {
    pub(crate) id: SignalId,
    marker: PhantomData<fn() -> T>,
}

/// An effect, kept so it can be disposed before its scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(pub(crate) ObserverId);

macro_rules! copy_handle {
    ($handle:ident) => {
        impl<T> Clone for $handle<T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<T> Copy for $handle<T> {}

        impl<T> PartialEq for $handle<T> {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        impl<T> Eq for $handle<T> {}

        impl<T> std::hash::Hash for $handle<T> {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        impl<T> std::fmt::Debug for $handle<T> {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}({:?})", stringify!($handle), self.id)
            }
        }

        impl<T> $handle<T> {
            pub(crate) fn from_id(id: SignalId) -> Self {
                Self {
                    id,
                    marker: PhantomData,
                }
            }
        }
    };
}

copy_handle!(Signal);
copy_handle!(Memo);
copy_handle!(Store);

impl<T: 'static> Signal<T> {
    /// Clones the value, subscribing the running observer.
    ///
    /// # Panics
    /// Panics when the signal's scope was disposed; use [`try_get`](Self::try_get) when that
    /// can happen.
    pub fn get(self, runtime: &mut Runtime) -> T
    where
        T: Clone,
    {
        self.try_get(runtime)
            .unwrap_or_else(|| panic!("{}", runtime.dead_handle_message(self.id)))
    }

    /// Clones the value, or `None` when the signal was disposed.
    pub fn try_get(self, runtime: &mut Runtime) -> Option<T>
    where
        T: Clone,
    {
        runtime.track(self.id);
        runtime.value::<T>(self.id).cloned()
    }

    /// Lends the value to `read`, subscribing the running observer.
    pub fn with<R>(self, runtime: &mut Runtime, read: impl FnOnce(&T) -> R) -> R {
        runtime.track(self.id);
        let value = runtime
            .value::<T>(self.id)
            .unwrap_or_else(|| panic!("{}", runtime.dead_handle_message(self.id)));
        read(value)
    }

    /// Lends the value without subscribing anything.
    pub fn with_untracked<R>(self, runtime: &Runtime, read: impl FnOnce(&T) -> R) -> R {
        let value = runtime
            .value::<T>(self.id)
            .unwrap_or_else(|| panic!("{}", runtime.dead_handle_message(self.id)));
        read(value)
    }

    /// Replaces the value and marks subscribers dirty.
    pub fn set(self, runtime: &mut Runtime, value: T) {
        self.update(runtime, |current| *current = value);
    }

    /// Replaces the value; returns `false` when the signal was disposed.
    pub fn try_set(self, runtime: &mut Runtime, value: T) -> bool {
        runtime.update_value(self.id, |current: &mut T| *current = value)
    }

    /// Mutates the value in place and marks subscribers dirty.
    pub fn update(self, runtime: &mut Runtime, change: impl FnOnce(&mut T)) {
        if !runtime.update_value(self.id, change) {
            panic!("{}", runtime.dead_handle_message(self.id));
        }
    }

    /// Mutates the value using another signal's value, avoiding a double borrow.
    pub fn update_from<S: 'static>(
        self,
        runtime: &mut Runtime,
        source: Signal<S>,
        change: impl FnOnce(&mut T, &S),
    ) {
        runtime.track(source.id);
        if !runtime.update_from(self.id, source.id, change) {
            panic!("{}", runtime.dead_handle_message(self.id));
        }
    }

    /// Whether the signal is still alive.
    pub fn is_alive(self, runtime: &Runtime) -> bool {
        runtime.value::<T>(self.id).is_some()
    }
}

impl<T: 'static> Memo<T> {
    /// Clones the current value, subscribing the running observer.
    pub fn get(self, runtime: &mut Runtime) -> T
    where
        T: Clone,
    {
        Signal::<T>::from_id(self.id).get(runtime)
    }

    /// Lends the current value, subscribing the running observer.
    pub fn with<R>(self, runtime: &mut Runtime, read: impl FnOnce(&T) -> R) -> R {
        Signal::<T>::from_id(self.id).with(runtime, read)
    }

    /// Clones the value, or `None` when the memo was disposed.
    pub fn try_get(self, runtime: &mut Runtime) -> Option<T>
    where
        T: Clone,
    {
        Signal::<T>::from_id(self.id).try_get(runtime)
    }
}

impl<T: 'static> Store<T> {
    /// Lends the value.
    pub fn with<R>(self, runtime: &Runtime, read: impl FnOnce(&T) -> R) -> R {
        let value = runtime
            .value::<T>(self.id)
            .unwrap_or_else(|| panic!("{}", runtime.dead_handle_message(self.id)));
        read(value)
    }

    /// Mutates the value without notifying anyone.
    pub fn update<R>(self, runtime: &mut Runtime, change: impl FnOnce(&mut T) -> R) -> R {
        match runtime.value_mut::<T>(self.id) {
            Some(value) => change(value),
            None => panic!("{}", runtime.dead_handle_message(self.id)),
        }
    }

    /// Mutates the value, or returns `None` when the store was disposed.
    pub fn try_update<R>(
        self,
        runtime: &mut Runtime,
        change: impl FnOnce(&mut T) -> R,
    ) -> Option<R> {
        runtime.value_mut::<T>(self.id).map(change)
    }
}
