//! Captures every `widgets::kit` preview: `scripts/capture-app.sh` style runs set the theme;
//! `CROWN_DESIGN_THEME=dark cargo run -p capture --example kit_previews`.

use capture::Carrier;

const PREVIEW_SIZE: (f32, f32) = (480.0, 320.0);

fn main() -> anyhow::Result<()> {
    let mut carrier = Carrier::new(
        "Kit",
        "Every crownui kit widget preview, captured headlessly from the current source.",
    );
    for preview in widgets::kit::previews() {
        let mut app = capture::mount((preview.build)(), PREVIEW_SIZE.0, PREVIEW_SIZE.1)?;
        carrier.capture_state(&mut app, "Kit", preview.name, "")?;
    }
    println!("{}", carrier.write(None)?.display());
    capture::finish()
}
