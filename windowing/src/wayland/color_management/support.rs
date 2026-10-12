use sctk::reexports::{
    client::WEnum,
    protocols::wp::color_management::v1::client::wp_color_manager_v1::{
        Feature, Primaries, RenderIntent, TransferFunction,
    },
};

/// What the color manager advertised before its `done` event, as far as tagging surfaces
/// sRGB needs to know.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColorSupport {
    parametric: bool,
    srgb_primaries: bool,
    gamma22: bool,
    srgb_transfer: bool,
    perceptual: bool,
}

impl ColorSupport {
    pub(crate) fn add_feature(&mut self, feature: WEnum<Feature>) {
        self.parametric |= feature == WEnum::Value(Feature::Parametric);
    }

    pub(crate) fn add_primaries(&mut self, primaries: WEnum<Primaries>) {
        self.srgb_primaries |= primaries == WEnum::Value(Primaries::Srgb);
    }

    pub(crate) fn add_transfer_function(&mut self, transfer_function: WEnum<TransferFunction>) {
        self.gamma22 |= transfer_function == WEnum::Value(TransferFunction::Gamma22);
        self.srgb_transfer |= transfer_function == WEnum::Value(TransferFunction::Srgb);
    }

    pub(crate) fn add_intent(&mut self, intent: WEnum<RenderIntent>) {
        self.perceptual |= intent == WEnum::Value(RenderIntent::Perceptual);
    }

    /// The transfer function for an sRGB description: `gamma22` as the protocol now
    /// recommends for desktop content, else the older `srgb`; `None` when a parametric sRGB
    /// description with perceptual intent cannot be built.
    pub(crate) fn srgb_transfer_function(self) -> Option<TransferFunction> {
        if !(self.parametric && self.srgb_primaries && self.perceptual) {
            return None;
        }
        if self.gamma22 {
            Some(TransferFunction::Gamma22)
        } else {
            self.srgb_transfer.then_some(TransferFunction::Srgb)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parametric_srgb() -> ColorSupport {
        let mut support = ColorSupport::default();
        support.add_feature(WEnum::Value(Feature::Parametric));
        support.add_primaries(WEnum::Value(Primaries::Srgb));
        support.add_intent(WEnum::Value(RenderIntent::Perceptual));
        support
    }

    #[test]
    fn gamma22_is_preferred_over_the_deprecated_srgb_curve() {
        let mut support = parametric_srgb();
        support.add_transfer_function(WEnum::Value(TransferFunction::Srgb));
        assert_eq!(
            support.srgb_transfer_function(),
            Some(TransferFunction::Srgb)
        );
        support.add_transfer_function(WEnum::Value(TransferFunction::Gamma22));
        assert_eq!(
            support.srgb_transfer_function(),
            Some(TransferFunction::Gamma22)
        );
    }

    #[test]
    fn icc_only_managers_are_left_alone() {
        let mut support = ColorSupport::default();
        support.add_feature(WEnum::Value(Feature::IccV2V4));
        support.add_transfer_function(WEnum::Value(TransferFunction::Gamma22));
        assert_eq!(support.srgb_transfer_function(), None);
    }
}
