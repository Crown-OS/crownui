use anyhow::{Context as _, Result};
use geometry::{CornerRadii, Point, Rect, Vector, rgb, rgba};
use graphics::{
    BoundaryId, Brush, CompositeOp, Composition, CreateRenderer, DrawCommand, DrawList, FontWeight,
    GradientStop, GraphicsBackend, LayerProps, Paint, Renderer, TextLayoutId, TextStyle,
    TextSystem,
};
use impeller::{ImpellerBackend, Offscreen, encode_png};

const LOGICAL_WIDTH: u32 = 480;
const LOGICAL_HEIGHT: u32 = 300;
const SCALE: f32 = 2.0;
const SCENE: BoundaryId = BoundaryId(1);
const TITLE: TextLayoutId = TextLayoutId(1);

fn diagonal_gradient(width: f32, height: f32) -> Paint {
    let stop = |offset, hex| GradientStop {
        offset,
        color: rgb(hex),
    };
    Paint {
        brush: Brush::LinearGradient {
            start: Point::new(0.0, 0.0),
            end: Point::new(width, height),
            stops: [stop(0.0, 0x1e1b4b), stop(1.0, 0x831843)]
                .into_iter()
                .collect(),
        },
        blur: None,
    }
}

fn scene() -> DrawList {
    let (width, height) = (LOGICAL_WIDTH as f32, LOGICAL_HEIGHT as f32);
    let mut list = DrawList::new();
    list.push(DrawCommand::FillRect(
        Rect::from_xywh(0.0, 0.0, width, height),
        diagonal_gradient(width, height),
    ));
    let radii = CornerRadii::uniform(20.0);
    for (index, color) in [0xf97316, 0x10b981].into_iter().enumerate() {
        let card = Rect::from_xywh(32.0 + index as f32 * 220.0, 40.0, 190.0, 120.0);
        list.shadow(card, radii, rgba(0x00000080), 28.0, Vector::new(0.0, 10.0));
        list.fill(card, radii, rgb(color));
    }
    list.text(TITLE, Point::new(52.0, 60.0));
    let glass = Rect::from_xywh(24.0, 130.0, width - 48.0, 120.0);
    list.save();
    list.clip(glass, radii);
    list.push(DrawCommand::SaveLayer {
        bounds: Some(glass),
        opacity: 1.0,
        backdrop_blur: Some(12.0),
    });
    list.fill(glass, CornerRadii::ZERO, rgba(0xffffff33));
    list.restore();
    list.stroke(glass, radii, 1.0, rgba(0xffffff99));
    list.restore();
    list
}

fn main() -> Result<()> {
    let output = std::env::args()
        .nth(1)
        .context("usage: render_png <output.png>")?;
    let offscreen = Offscreen {
        width: (LOGICAL_WIDTH as f32 * SCALE) as u32,
        height: (LOGICAL_HEIGHT as f32 * SCALE) as u32,
        scale: SCALE,
    };
    let mut renderer = ImpellerBackend::create_renderer(&offscreen)?;
    let mut text = ImpellerBackend::create_text_system()?;
    let title_style = TextStyle {
        font_size: 20.0,
        font_weight: FontWeight::SEMIBOLD,
        color: rgb(0xffffff),
        ..TextStyle::default()
    };
    renderer.upload_text(
        TITLE,
        text.layout("Rendered offscreen", &title_style, Some(160.0)),
    );
    renderer.record_boundary(SCENE, &scene());
    renderer.render(&Composition {
        clear_color: rgb(0x000000),
        ops: vec![
            CompositeOp::PushLayer(LayerProps::default()),
            CompositeOp::Draw(SCENE),
            CompositeOp::PopLayer,
        ],
    })?;
    let pixels = renderer.read_pixels()?;
    std::fs::write(&output, encode_png(&pixels)?).with_context(|| format!("writing {output}"))?;
    eprintln!(
        "[render_png] {:?} rendered {}x{} @{SCALE} into {output}",
        renderer.api(),
        pixels.width,
        pixels.height
    );
    Ok(())
}
