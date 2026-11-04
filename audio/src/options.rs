use crate::{Enhancement, SourceId};

/// What [`Microphone::open`](crate::Microphone::open) captures: the default source with
/// [`Enhancement::NoiseSuppression`] unless told otherwise.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MicrophoneOptions {
    pub(crate) source: Option<SourceId>,
    pub(crate) enhancement: Enhancement,
}

impl MicrophoneOptions {
    /// Captures `source` instead of the system default.
    pub fn source(self, source: SourceId) -> Self {
        Self {
            source: Some(source),
            ..self
        }
    }

    /// Asks crown-voiced for `enhancement`; [`Enhancement::Off`] captures the raw source.
    pub fn enhancement(self, enhancement: Enhancement) -> Self {
        Self {
            enhancement,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_the_default_source_with_noise_suppression() {
        let options = MicrophoneOptions::default();
        assert_eq!(options.source, None);
        assert_eq!(options.enhancement, Enhancement::NoiseSuppression);
    }

    #[test]
    fn builders_set_source_and_enhancement() {
        let options = MicrophoneOptions::default()
            .enhancement(Enhancement::Off)
            .source(SourceId::new("alsa_input.usb"));
        assert_eq!(options.source, Some(SourceId::new("alsa_input.usb")));
        assert_eq!(options.enhancement, Enhancement::Off);
    }
}
