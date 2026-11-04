use reactive::Signal;
use tokio::sync::watch;
use view::Cx;

/// Keeps `signal` equal to the latest value `receiver` sees while the calling component is
/// mounted, re-arming one `cx.spawn` per change; stops when the sender is gone.
pub(super) fn follow_watch<T: Clone + Send + Sync + 'static>(
    cx: &mut Cx,
    signal: Signal<T>,
    mut receiver: watch::Receiver<T>,
) {
    let next_change = async move {
        let open = receiver.changed().await.is_ok();
        (receiver, open)
    };
    cx.spawn(next_change, move |cx, (receiver, open)| {
        if open {
            signal.set(cx, receiver.borrow().clone());
            follow_watch(cx, signal, receiver);
        }
    });
}
