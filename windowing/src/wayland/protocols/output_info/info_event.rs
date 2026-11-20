use crownos_protocols::output_info::v1::client::crownos_output_info_v1::{
    Capability, Event, State,
};

use super::{CrownOutput, ShellSurfaces};
use crate::wayland::util::bitfield::known_flags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InfoBatch {
    Pending,
    Done,
}

/// Folds one event into `received`, which keeps every field a later batch does not resend.
pub(crate) fn apply_info_event(received: &mut CrownOutput, event: Event) -> InfoBatch {
    match event {
        Event::Identity { identity } => received.identity = identity,
        Event::Shell { shell } => {
            received.shell = ShellSurfaces::from_protocol_bits(known_flags(shell).bits());
        }
        Event::State { state } => {
            let state = known_flags(state);
            received.primary = state.contains(State::Primary);
            received.focused = state.contains(State::Focused);
        }
        Event::Capabilities { capabilities } => {
            let capabilities = known_flags(capabilities);
            received.hdr_capable = capabilities.contains(Capability::Hdr);
            received.virtual_output = capabilities.contains(Capability::Virtual);
        }
        Event::ScanoutDepth { bits } => received.scanout_bits = bits,
        Event::Done => return InfoBatch::Done,
        _ => {}
    }
    InfoBatch::Pending
}

#[cfg(test)]
mod tests {
    use crownos_protocols::output_info::v1::client::crownos_output_info_v1::Shell;
    use sctk::reexports::client::WEnum;

    use super::*;

    fn apply_all(received: &mut CrownOutput, events: Vec<Event>) -> Vec<InfoBatch> {
        events
            .into_iter()
            .map(|event| apply_info_event(received, event))
            .collect()
    }

    #[test]
    fn the_first_burst_describes_the_whole_output() {
        let mut received = CrownOutput::default();
        let batches = apply_all(
            &mut received,
            vec![
                Event::Identity {
                    identity: "Dell Inc. U2720Q 7XK2".into(),
                },
                Event::Shell {
                    shell: WEnum::Value(Shell::Bar | Shell::Dock),
                },
                Event::State {
                    state: WEnum::Value(State::Primary),
                },
                Event::Capabilities {
                    capabilities: WEnum::Value(Capability::Hdr),
                },
                Event::ScanoutDepth { bits: 10 },
                Event::Done,
            ],
        );
        assert_eq!(batches.last(), Some(&InfoBatch::Done));
        assert_eq!(
            received,
            CrownOutput {
                identity: "Dell Inc. U2720Q 7XK2".into(),
                shell: ShellSurfaces::BAR | ShellSurfaces::DOCK,
                primary: true,
                focused: false,
                hdr_capable: true,
                virtual_output: false,
                scanout_bits: 10,
            }
        );
    }

    #[test]
    fn a_later_batch_changes_only_the_fields_it_sends() {
        let mut received = CrownOutput {
            identity: "HDMI-A-1".into(),
            shell: ShellSurfaces::BAR,
            primary: true,
            scanout_bits: 8,
            ..CrownOutput::default()
        };
        apply_all(
            &mut received,
            vec![
                Event::State {
                    state: WEnum::Value(State::Primary | State::Focused),
                },
                Event::Done,
            ],
        );
        assert_eq!(received.identity, "HDMI-A-1");
        assert_eq!(received.shell, ShellSurfaces::BAR);
        assert!(received.primary && received.focused);
        assert_eq!(received.scanout_bits, 8);
    }

    #[test]
    fn flags_from_a_newer_compositor_are_ignored() {
        let mut received = CrownOutput::default();
        apply_info_event(
            &mut received,
            Event::Capabilities {
                capabilities: WEnum::Unknown(2 | 64),
            },
        );
        assert!(received.virtual_output);
        assert!(!received.hdr_capable);
    }
}
