# reactive

Fine-grained reactivity without `Rc`, `RefCell` or locks. The [`Runtime`] owns every value;
handles (`Signal<T>`, `Memo<T>`, `Store<T>`) are `Copy` keys into it, and every read or write
takes `&mut Runtime` (UI contexts deref to it).

```rust
use reactive::Runtime;

let mut runtime = Runtime::new();
let count = runtime.signal(1);
let doubled = runtime.memo(move |rt| count.get(rt) * 2);
let seen = runtime.store(Vec::new());
runtime.effect(move |rt| {
    let value = doubled.get(rt);
    seen.update(rt, |log| log.push(value));
});
count.set(&mut runtime, 5);
runtime.flush().unwrap();
assert_eq!(seen.with(&runtime, |log| log.clone()), vec![2, 10]);
```

- **Scopes** own signals, observers, stores, cleanups and child scopes; `dispose_scope` frees all.
- **Flush** is glitch-free: observers run in height order, memos stop propagation when their
  value is unchanged, writes made by effects run in the next pass, and a pass cap reports cycles.
- **Environment**: `provide` puts a value on the current scope; `environment` finds the nearest.
- **Panics** in observers are caught, the observer is quarantined, and the nearest error boundary
  scope receives the message.
- **Borrowing**: `with` lends `&T` while the runtime is borrowed, so read other signals first
  (or use `with2`).

Non-goals: async (the view layer posts results back as closures), threads (the runtime is
single-threaded by construction).
