use windowing::wayland::{Output, OutputEvent, OutputId};

/// A monitor as [`AnnouncedOutputs`] keeps it.
pub(crate) trait Announced {
    type Id: Copy + PartialEq;

    fn id(&self) -> Self::Id;
}

impl Announced for Output {
    type Id = OutputId;

    fn id(&self) -> OutputId {
        self.id
    }
}

/// The monitors in the order the platform announced them, and whether the surfaces placed on them
/// are out of date. Changes and wakes only mark them so; the runner places surfaces once, after
/// every event of a dispatch is in.
pub(crate) struct AnnouncedOutputs<Monitor: Announced> {
    outputs: Vec<Monitor>,
    out_of_date: bool,
}

impl<Monitor: Announced> Default for AnnouncedOutputs<Monitor> {
    fn default() -> Self {
        Self {
            outputs: Vec::new(),
            out_of_date: false,
        }
    }
}

impl<Monitor: Announced> AnnouncedOutputs<Monitor> {
    /// Adds a new monitor at the end, or replaces a known one where it is.
    pub(crate) fn update(&mut self, output: Monitor) {
        match self.position(output.id()) {
            Some(index) => self.outputs[index] = output,
            None => self.outputs.push(output),
        }
        self.out_of_date = true;
    }

    pub(crate) fn remove(&mut self, id: Monitor::Id) {
        self.outputs.retain(|output| output.id() != id);
        self.out_of_date = true;
    }

    /// Something outside the monitors may change where surfaces go.
    pub(crate) fn wake(&mut self) {
        self.out_of_date = true;
    }

    /// Every monitor, once after anything changed since the last call.
    pub(crate) fn take_out_of_date(&mut self) -> Option<&[Monitor]> {
        std::mem::take(&mut self.out_of_date).then_some(self.outputs.as_slice())
    }

    pub(crate) fn get(&self, id: Monitor::Id) -> Option<&Monitor> {
        self.position(id).map(|index| &self.outputs[index])
    }

    fn position(&self, id: Monitor::Id) -> Option<usize> {
        self.outputs.iter().position(|output| output.id() == id)
    }
}

impl AnnouncedOutputs<Output> {
    pub(crate) fn apply(&mut self, event: &OutputEvent) {
        match event {
            OutputEvent::Added(output) | OutputEvent::Changed(output) => {
                self.update(output.clone());
            }
            OutputEvent::Removed(id) => self.remove(*id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct FakeOutput {
        id: u32,
        focused: bool,
    }

    impl Announced for FakeOutput {
        type Id = u32;

        fn id(&self) -> u32 {
            self.id
        }
    }

    fn output(id: u32, focused: bool) -> FakeOutput {
        FakeOutput { id, focused }
    }

    fn announced(ids: &[u32]) -> AnnouncedOutputs<FakeOutput> {
        let mut outputs = AnnouncedOutputs::default();
        for &id in ids {
            outputs.update(output(id, false));
        }
        outputs
    }

    #[test]
    fn two_changes_in_one_dispatch_are_placed_once_with_both() {
        let mut outputs = announced(&[1, 2]);
        let _ = outputs.take_out_of_date();
        outputs.update(output(1, false));
        outputs.update(output(2, true));
        assert_eq!(
            outputs.take_out_of_date(),
            Some([output(1, false), output(2, true)].as_slice())
        );
        assert_eq!(outputs.take_out_of_date(), None);
    }

    #[test]
    fn nothing_is_placed_until_something_changes_or_wakes() {
        let mut outputs = announced(&[]);
        assert_eq!(outputs.take_out_of_date(), None);
        outputs.wake();
        outputs.wake();
        assert_eq!(outputs.take_out_of_date(), Some([].as_slice()));
        assert_eq!(outputs.take_out_of_date(), None);
    }

    #[test]
    fn monitors_keep_the_order_they_were_announced_in() {
        let mut outputs = announced(&[3, 1, 2]);
        outputs.remove(1);
        outputs.update(output(3, true));
        outputs.update(output(4, false));
        let ids: Vec<u32> = outputs
            .take_out_of_date()
            .into_iter()
            .flatten()
            .map(|output| output.id)
            .collect();
        assert_eq!(ids, [3, 2, 4]);
        assert_eq!(outputs.get(3), Some(&output(3, true)));
    }
}
