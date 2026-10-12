use crownos_config::Subscription;
use serde::de::DeserializeOwned;
use tokio::sync::watch;

/// The latest value of a config section, updated whenever its file changes on disk.
///
/// Dropping the watch unregisters it.
pub struct SectionWatch<T> {
    receiver: watch::Receiver<T>,
    _subscriptions: Vec<Subscription>,
}

impl<T: DeserializeOwned + Send + Sync + 'static> SectionWatch<T> {
    pub(crate) fn subscribe(section: &str, initial: T) -> Self {
        let (sender, receiver) = watch::channel(initial);
        let subscription = crownos_config::subscribe_typed::<T, _>(section, move |value| {
            sender.send_replace(value);
        });
        Self::from_parts(receiver, vec![subscription])
    }
}

impl<T> SectionWatch<T> {
    pub(crate) fn from_parts(
        receiver: watch::Receiver<T>,
        subscriptions: Vec<Subscription>,
    ) -> Self {
        Self {
            receiver,
            _subscriptions: subscriptions,
        }
    }
}

impl<T: Clone> SectionWatch<T> {
    /// The value as of now.
    pub fn current(&self) -> T {
        self.receiver.borrow().clone()
    }

    /// Waits for the next edit and returns the new value.
    pub async fn changed(&mut self) -> T {
        let _ = self.receiver.changed().await;
        self.receiver.borrow_and_update().clone()
    }
}
