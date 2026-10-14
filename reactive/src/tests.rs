use crate::{FlushError, Runtime};

#[test]
fn effects_rerun_when_their_signals_change() {
    let mut runtime = Runtime::new();
    let count = runtime.signal(1);
    let seen = runtime.store(Vec::new());
    runtime.effect(move |rt| {
        let value = count.get(rt);
        seen.update(rt, |log| log.push(value));
    });
    count.set(&mut runtime, 2);
    count.set(&mut runtime, 3);
    assert_eq!(runtime.flush().map(|stats| stats.observers_run), Ok(1));
    assert_eq!(seen.with(&runtime, Clone::clone), vec![1, 3]);
}

#[test]
fn unchanged_memos_stop_propagation() {
    let mut runtime = Runtime::new();
    let number = runtime.signal(2);
    let is_even = runtime.memo(move |rt| number.get(rt) % 2 == 0);
    let runs = runtime.store(0);
    runtime.effect(move |rt| {
        is_even.get(rt);
        runs.update(rt, |runs| *runs += 1);
    });
    number.set(&mut runtime, 4);
    assert!(runtime.flush().is_ok());
    assert_eq!(runs.with(&runtime, |runs| *runs), 1);
    number.set(&mut runtime, 5);
    assert!(runtime.flush().is_ok());
    assert_eq!(runs.with(&runtime, |runs| *runs), 2);
}

#[test]
fn diamonds_are_glitch_free() {
    let mut runtime = Runtime::new();
    let base = runtime.signal(1);
    let plus_one = runtime.memo(move |rt| base.get(rt) + 1);
    let doubled = runtime.memo(move |rt| base.get(rt) * 2);
    let seen = runtime.store(Vec::new());
    runtime.effect(move |rt| {
        let pair = (plus_one.get(rt), doubled.get(rt));
        seen.update(rt, |log| log.push(pair));
    });
    base.set(&mut runtime, 10);
    assert!(runtime.flush().is_ok());
    assert_eq!(seen.with(&runtime, Clone::clone), vec![(2, 2), (11, 20)]);
}

#[test]
fn effects_that_feed_themselves_are_reported_as_cycles() {
    let mut runtime = Runtime::new();
    let counter = runtime.signal(0_u32);
    runtime.effect(move |rt| {
        let value = counter.get(rt);
        counter.set(rt, value + 1);
    });
    counter.set(&mut runtime, 5);
    assert!(matches!(runtime.flush(), Err(FlushError::Cycle { .. })));
    assert!(!runtime.has_pending());
}

#[test]
fn writes_made_by_effects_run_in_the_next_pass() {
    let mut runtime = Runtime::new();
    let source = runtime.signal(1);
    let mirror = runtime.signal(0);
    runtime.effect(move |rt| {
        let value = source.get(rt);
        mirror.set(rt, value * 10);
    });
    let seen = runtime.store(Vec::new());
    runtime.effect(move |rt| {
        let value = mirror.get(rt);
        seen.update(rt, |log| log.push(value));
    });
    assert!(runtime.flush().is_ok());
    source.set(&mut runtime, 2);
    assert_eq!(runtime.flush().map(|stats| stats.passes), Ok(2));
    assert_eq!(seen.with(&runtime, Clone::clone), vec![10, 20]);
}

#[test]
fn disposing_a_scope_frees_its_nodes_and_runs_cleanups() {
    let mut runtime = Runtime::new();
    let cleaned = runtime.store(false);
    let scope = runtime.create_scope();
    let signal = runtime.with_scope(scope, |rt| {
        rt.on_cleanup(move |rt| cleaned.update(rt, |flag| *flag = true));
        let signal = rt.signal(7);
        rt.effect(move |rt| {
            signal.get(rt);
        });
        signal
    });
    let observers_before = runtime.observer_count();
    runtime.dispose_scope(scope);
    assert!(cleaned.with(&runtime, |flag| *flag));
    assert_eq!(runtime.observer_count(), observers_before - 1);
    assert_eq!(signal.try_get(&mut runtime), None);
    assert!(!signal.try_set(&mut runtime, 1));
    assert!(!runtime.is_scope_alive(scope));
}

#[test]
fn stale_handles_panic_with_a_diagnostic_code() {
    let mut runtime = Runtime::new();
    let scope = runtime.create_scope();
    let signal = runtime.with_scope(scope, |rt| rt.signal(1));
    runtime.dispose_scope(scope);
    let outcome =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| signal.get(&mut runtime)));
    let Err(payload) = outcome else {
        panic!("reading a disposed signal must panic");
    };
    let message = payload
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_default();
    assert!(message.starts_with("CUI0001"), "{message}");
}

#[test]
fn environment_values_come_from_the_nearest_scope() {
    let mut runtime = Runtime::new();
    runtime.provide("root theme");
    let child = runtime.create_scope();
    let grandchild = runtime.with_scope(child, |rt| {
        rt.provide("child theme");
        rt.create_scope()
    });
    let found = runtime.with_scope(grandchild, |rt| rt.environment::<&str>());
    assert_eq!(
        found.map(|theme| theme.get(&mut runtime)),
        Some("child theme")
    );
    assert!(runtime.environment::<u32>().is_none());
}

#[test]
fn panicking_observers_are_quarantined_and_reported_to_the_boundary() {
    let mut runtime = Runtime::new();
    let reports = runtime.store(Vec::new());
    let boundary = runtime.create_scope();
    runtime.set_error_boundary(boundary, move |rt, panic| {
        let message = panic.message.clone();
        reports.update(rt, |log| log.push(message));
    });
    let trigger = runtime.signal(false);
    let inner = runtime.create_child_scope(boundary);
    runtime.with_scope(inner, |rt| {
        rt.effect(move |rt| {
            if trigger.get(rt) {
                panic!("render failed");
            }
        });
    });
    trigger.set(&mut runtime, true);
    assert!(runtime.flush().is_ok());
    trigger.set(&mut runtime, false);
    trigger.set(&mut runtime, true);
    assert!(runtime.flush().is_ok());
    assert_eq!(
        reports.with(&runtime, Clone::clone),
        vec!["render failed".to_owned()]
    );
    assert!(runtime.take_panics().is_empty());
}

#[test]
fn dependencies_follow_the_branch_taken() {
    let mut runtime = Runtime::new();
    let use_first = runtime.signal(true);
    let first = runtime.signal(1);
    let second = runtime.signal(2);
    let runs = runtime.store(0);
    runtime.effect(move |rt| {
        let _ = if use_first.get(rt) {
            first.get(rt)
        } else {
            second.get(rt)
        };
        runs.update(rt, |runs| *runs += 1);
    });
    use_first.set(&mut runtime, false);
    assert!(runtime.flush().is_ok());
    first.set(&mut runtime, 10);
    assert!(runtime.flush().is_ok());
    assert_eq!(runs.with(&runtime, |runs| *runs), 2);
}

#[test]
fn two_signals_can_be_read_together_and_combined() {
    let mut runtime = Runtime::new();
    let total = runtime.signal(1);
    let step = runtime.signal(5);
    total.update_from(&mut runtime, step, |total, step| *total += step);
    assert_eq!(runtime.with2(total, step, |total, step| total * step), 30);
}

#[test]
fn untracked_reads_do_not_subscribe() {
    let mut runtime = Runtime::new();
    let watched = runtime.signal(0);
    let ignored = runtime.signal(0);
    let runs = runtime.store(0);
    runtime.effect(move |rt| {
        watched.get(rt);
        rt.untrack(|rt| ignored.get(rt));
        runs.update(rt, |runs| *runs += 1);
    });
    ignored.set(&mut runtime, 1);
    assert!(runtime.flush().is_ok());
    assert_eq!(runs.with(&runtime, |runs| *runs), 1);
}
