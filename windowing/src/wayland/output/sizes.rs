use super::OutputId;
use crate::wayland::event_loop::WaylandPlatform;

/// The logical size of every output that reported one, in the order the compositor announced them.
pub(crate) struct OutputSizes(Vec<(OutputId, (u32, u32))>);

impl OutputSizes {
    /// The size of `output`, or of the first output when `output` is unknown or has no size yet.
    pub(crate) fn size_of(&self, output: Option<OutputId>) -> Option<(u32, u32)> {
        output
            .and_then(|wanted| self.0.iter().find(|(id, _)| *id == wanted))
            .or(self.0.first())
            .map(|&(_, size)| size)
    }
}

impl WaylandPlatform {
    pub(crate) fn output_sizes(&self) -> OutputSizes {
        let sized = self.outputs().filter_map(|output| {
            let (width, height) = output.logical_size?;
            let size = (u32::try_from(width).ok()?, u32::try_from(height).ok()?);
            (size.0 > 0 && size.1 > 0).then_some((output.id, size))
        });
        OutputSizes(sized.collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sizes() -> OutputSizes {
        OutputSizes(vec![
            (OutputId(1), (1920, 1080)),
            (OutputId(2), (2560, 1440)),
        ])
    }

    #[test]
    fn a_surface_takes_the_size_of_its_own_output() {
        assert_eq!(sizes().size_of(Some(OutputId(2))), Some((2560, 1440)));
    }

    #[test]
    fn an_unknown_or_missing_output_falls_back_to_the_first() {
        assert_eq!(sizes().size_of(Some(OutputId(9))), Some((1920, 1080)));
        assert_eq!(sizes().size_of(None), Some((1920, 1080)));
        assert_eq!(OutputSizes(Vec::new()).size_of(None), None);
    }
}
