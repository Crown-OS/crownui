/// What to tell the app about an output after something happened to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputReport {
    Added,
    Changed,
    Silent,
}

/// Holds an output back from the app until the compositor described it, so a surface is never
/// built for it on a guess and then torn down.
#[derive(Debug, Default)]
pub(crate) struct Announcement {
    announced: bool,
}

impl Announcement {
    pub(crate) fn info_done(&mut self) -> OutputReport {
        if std::mem::replace(&mut self.announced, true) {
            OutputReport::Changed
        } else {
            OutputReport::Added
        }
    }

    /// The compositor answered a roundtrip sent after the info request; an output it did not
    /// describe by then is announced without its info.
    pub(crate) fn deadline_passed(&mut self) -> OutputReport {
        if std::mem::replace(&mut self.announced, true) {
            OutputReport::Silent
        } else {
            OutputReport::Added
        }
    }

    pub(crate) fn output_updated(&self) -> OutputReport {
        if self.announced {
            OutputReport::Changed
        } else {
            OutputReport::Silent
        }
    }

    pub(crate) fn reports_removal(&self) -> bool {
        self.announced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_done_adds_the_output_and_later_ones_change_it() {
        let mut announcement = Announcement::default();
        assert_eq!(announcement.info_done(), OutputReport::Added);
        assert_eq!(announcement.deadline_passed(), OutputReport::Silent);
        assert_eq!(announcement.info_done(), OutputReport::Changed);
    }

    #[test]
    fn a_deadline_before_any_done_adds_it_without_info() {
        let mut announcement = Announcement::default();
        assert_eq!(announcement.deadline_passed(), OutputReport::Added);
        assert_eq!(announcement.info_done(), OutputReport::Changed);
    }

    #[test]
    fn output_updates_wait_for_the_announcement() {
        let mut announcement = Announcement::default();
        assert_eq!(announcement.output_updated(), OutputReport::Silent);
        announcement.info_done();
        assert_eq!(announcement.output_updated(), OutputReport::Changed);
    }

    #[test]
    fn an_output_gone_before_its_announcement_is_never_removed() {
        let mut announcement = Announcement::default();
        assert!(!announcement.reports_removal());
        announcement.deadline_passed();
        assert!(announcement.reports_removal());
    }
}
