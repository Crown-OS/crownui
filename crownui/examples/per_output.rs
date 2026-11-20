use std::hash::{DefaultHasher, Hash, Hasher};

use crownui::{
    prelude::*,
    surfaces::{Output, Surfaces},
};

struct OutputLabels;

impl Surfaces for OutputLabels {
    fn placement(&mut self, output: &Output) -> Option<u64> {
        let wanted = output.crown.as_ref().is_none_or(|crown| crown.shell.bar());
        wanted.then(|| label_key(output))
    }

    fn build(&mut self, output: &Output) -> Vec<(AnyView, AppConfig)> {
        let config = AppConfig::new("Per output")
            .clear_color(rgba(0x0f172ae6))
            .layer(
                LayerOptions::new("per-output", Anchor::TOP | Anchor::RIGHT)
                    .width(LayerExtent::Fixed(320))
                    .height(LayerExtent::Fixed(104))
                    .margin(LayerMargin {
                        top: 16,
                        right: 16,
                        ..LayerMargin::default()
                    }),
            );
        vec![(label(output).boxed(), config)]
    }
}

fn label_key(output: &Output) -> u64 {
    let mut hasher = DefaultHasher::new();
    let shown = output
        .crown
        .as_ref()
        .map(|crown| (&crown.identity, crown.primary, crown.focused));
    (&output.name, shown).hash(&mut hasher);
    hasher.finish()
}

fn label(output: &Output) -> impl View {
    let name = output
        .name
        .clone()
        .unwrap_or_else(|| "unnamed output".into());
    let details = match &output.crown {
        Some(crown) => format!(
            "{}\nprimary: {}  focused: {}",
            crown.identity, crown.primary, crown.focused
        ),
        None => "no crownos_output_info_v1".into(),
    };
    vstack((
        text(name).text_size(18.0).font_semibold().text_color(WHITE),
        text(details).text_size(13.0).text_color(rgb(0xcbd5e1)),
    ))
    .p(16.0)
    .gap(6.0)
}

fn main() -> Result<(), crownui::RunError> {
    tracing_subscriber::fmt::init();
    crownui::run_surfaces(OutputLabels)
}
