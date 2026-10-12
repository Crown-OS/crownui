use geometry::{CornerRadii, CornerShape, Point, Rect, WHITE, rgb};
use graphics::{
    BoundaryId, CompositeOp, Composition, CreateRenderer, DrawCommand, DrawList, LayerProps, Paint,
    Path, Renderer,
};
use impeller::{ImpellerBackend, ImpellerRenderer, Offscreen, Pixels, RenderApi, encode_png};

const RED: [u8; 4] = [255, 0, 0, 255];
const OPAQUE_WHITE: [u8; 4] = [255, 255, 255, 255];
const CARD: BoundaryId = BoundaryId(1);

fn assert_pixel(pixels: &Pixels, x: u32, y: u32, expected: [u8; 4]) {
    let Some(actual) = pixels.pixel(x, y) else {
        panic!("({x}, {y}) is outside {}x{}", pixels.width, pixels.height);
    };
    let close = actual
        .iter()
        .zip(expected)
        .all(|(actual, expected)| actual.abs_diff(expected) <= 2);
    assert!(
        close,
        "pixel ({x}, {y}) is {actual:?}, expected {expected:?}"
    );
}

fn card_frame() -> Composition {
    Composition {
        clear_color: WHITE,
        ops: vec![
            CompositeOp::PushLayer(LayerProps::default()),
            CompositeOp::Draw(CARD),
            CompositeOp::PopLayer,
        ],
    }
}

#[test]
fn offscreen_frames_read_back_top_down_at_device_scale() {
    let offscreen = Offscreen {
        width: 128,
        height: 96,
        scale: 2.0,
    };
    let mut renderer = match ImpellerBackend::create_renderer(&offscreen) {
        Ok(renderer) => renderer,
        Err(error) => panic!("offscreen GLES renderer: {error}"),
    };
    assert!(matches!(
        renderer.api(),
        RenderApi::GlesSurfaceless | RenderApi::GlesPbuffer
    ));

    let mut card = DrawList::new();
    card.fill(
        Rect::from_xywh(4.0, 4.0, 40.0, 32.0),
        CornerRadii::uniform(6.0),
        rgb(0xff0000),
    );
    renderer.record_boundary(CARD, &card);
    if let Err(error) = renderer.render(&card_frame()) {
        panic!("offscreen render: {error}");
    }
    let pixels = match renderer.read_pixels() {
        Ok(pixels) => pixels,
        Err(error) => panic!("read_pixels: {error}"),
    };
    assert_eq!((pixels.width, pixels.height), (128, 96));
    assert_eq!(renderer.frames_presented(), 1);
    assert_pixel(&pixels, 48, 40, RED);
    assert_pixel(&pixels, 40, 14, RED);
    assert_pixel(&pixels, 40, 80, OPAQUE_WHITE);
    assert_pixel(&pixels, 0, 0, OPAQUE_WHITE);
    assert_pixel(&pixels, 127, 95, OPAQUE_WHITE);
    assert!(encode_png(&pixels).is_ok_and(|png| png.starts_with(b"\x89PNG")));

    renderer.set_viewport(32, 16, 1.0);
    if let Err(error) = renderer.render(&card_frame()) {
        panic!("render after resize: {error}");
    }
    let resized = match renderer.read_pixels() {
        Ok(pixels) => pixels,
        Err(error) => panic!("read_pixels after resize: {error}"),
    };
    assert_eq!((resized.width, resized.height), (32, 16));
    assert_pixel(&resized, 20, 10, RED);
    assert_pixel(&resized, 2, 2, OPAQUE_WHITE);
}

#[test]
fn offscreen_edges_are_antialiased() {
    let offscreen = Offscreen {
        width: 64,
        height: 64,
        scale: 1.0,
    };
    let mut renderer = match ImpellerBackend::create_renderer(&offscreen) {
        Ok(renderer) => renderer,
        Err(error) => panic!("offscreen GLES renderer: {error}"),
    };
    let triangle = Path::new()
        .move_to(Point::new(0.0, 0.0))
        .line_to(Point::new(64.0, 0.0))
        .line_to(Point::new(0.0, 64.0))
        .close();
    let mut card = DrawList::new();
    card.push(DrawCommand::FillPath(triangle, Paint::solid(rgb(0xff0000))));
    renderer.record_boundary(CARD, &card);
    if let Err(error) = renderer.render(&card_frame()) {
        panic!("offscreen render: {error}");
    }
    let pixels = match renderer.read_pixels() {
        Ok(pixels) => pixels,
        Err(error) => panic!("read_pixels: {error}"),
    };
    assert_pixel(&pixels, 8, 8, RED);
    assert_pixel(&pixels, 56, 56, OPAQUE_WHITE);
    let Some([_, half_covered_green, ..]) = pixels.pixel(32, 31) else {
        panic!("(32, 31) is outside {}x{}", pixels.width, pixels.height);
    };
    assert!(
        (30..=225).contains(&half_covered_green),
        "the diagonal edge pixel is not blended (green {half_covered_green})"
    );
}

fn renderer(offscreen: &Offscreen, corners: CornerShape) -> ImpellerRenderer {
    let mut renderer = match ImpellerBackend::create_renderer(offscreen) {
        Ok(renderer) => renderer,
        Err(error) => panic!("offscreen GLES renderer: {error}"),
    };
    renderer.set_corner_shape(corners);
    renderer
}

fn draw(renderer: &mut ImpellerRenderer, list: &DrawList, frame: &Composition) -> Pixels {
    renderer.record_boundary(CARD, list);
    if let Err(error) = renderer.render(frame) {
        panic!("offscreen render: {error}");
    }
    match renderer.read_pixels() {
        Ok(pixels) => pixels,
        Err(error) => panic!("read_pixels: {error}"),
    }
}

/// The most coverage levels 8x MSAA can produce: zero to eight samples.
const MSAA_LEVELS: usize = 9;

/// Distinct green levels of red-on-white pixels in a device-pixel square.
fn coverage_levels(pixels: &Pixels, from: (u32, u32), to: (u32, u32)) -> usize {
    let mut levels: Vec<u8> = (from.1..to.1)
        .flat_map(|y| (from.0..to.0).map(move |x| (x, y)))
        .filter_map(|(x, y)| pixels.pixel(x, y).map(|[_, green, ..]| green))
        .collect();
    levels.sort_unstable();
    levels.dedup();
    levels.len()
}

#[test]
fn rounded_corners_have_analytic_coverage() {
    let offscreen = Offscreen {
        width: 160,
        height: 120,
        scale: 2.0,
    };
    let mut card = DrawList::new();
    card.fill(
        Rect::from_xywh(4.3, 4.6, 64.0, 40.0),
        CornerRadii::uniform(12.0),
        rgb(0xff0000),
    );
    for corners in [CornerShape::Round, CornerShape::Squircle] {
        let pixels = draw(&mut renderer(&offscreen, corners), &card, &card_frame());
        assert_pixel(&pixels, 70, 50, RED);
        assert_pixel(&pixels, 9, 10, OPAQUE_WHITE);
        assert_pixel(&pixels, 150, 110, OPAQUE_WHITE);
        let levels = coverage_levels(&pixels, (8, 9), (40, 40));
        assert!(
            levels > MSAA_LEVELS,
            "{corners:?} corner has only {levels} coverage levels"
        );
    }
}

#[test]
#[ignore = "NVIDIA's EGL segfaults freeing a context Impeller drew an offscreen layer with; \
            run with __EGL_VENDOR_LIBRARY_FILENAMES pointing at Mesa and --ignored"]
fn rounded_clips_cut_smoothly() {
    let offscreen = Offscreen {
        width: 128,
        height: 128,
        scale: 1.0,
    };
    let mut card = DrawList::new();
    card.fill(
        Rect::from_xywh(0.0, 0.0, 128.0, 128.0),
        CornerRadii::ZERO,
        rgb(0xff0000),
    );
    for (opacity, centre) in [(0.5, [255, 128, 128, 255]), (1.0, RED)] {
        let frame = Composition {
            clear_color: WHITE,
            ops: vec![
                CompositeOp::PushLayer(LayerProps {
                    opacity,
                    clip: Some((
                        Rect::from_xywh(16.0, 16.0, 96.0, 96.0),
                        CornerRadii::uniform(24.0),
                    )),
                    ..LayerProps::default()
                }),
                CompositeOp::Draw(CARD),
                CompositeOp::PopLayer,
            ],
        };
        let pixels = draw(&mut renderer(&offscreen, CornerShape::Round), &card, &frame);
        assert_pixel(&pixels, 64, 64, centre);
        assert_pixel(&pixels, 17, 17, OPAQUE_WHITE);
        assert_pixel(&pixels, 8, 64, OPAQUE_WHITE);
        let levels = coverage_levels(&pixels, (16, 16), (40, 40));
        assert!(
            levels > MSAA_LEVELS,
            "the corner clipped at opacity {opacity} has only {levels} coverage levels"
        );
    }
}

#[test]
fn square_stroke_corners_stay_mitred() {
    let offscreen = Offscreen {
        width: 64,
        height: 64,
        scale: 1.0,
    };
    let mut card = DrawList::new();
    card.stroke(
        Rect::from_xywh(10.0, 10.0, 40.0, 30.0),
        CornerRadii::ZERO,
        4.0,
        rgb(0xff0000),
    );
    let pixels = draw(
        &mut renderer(&offscreen, CornerShape::Round),
        &card,
        &card_frame(),
    );
    assert_pixel(&pixels, 8, 8, RED);
    assert_pixel(&pixels, 51, 41, RED);
    assert_pixel(&pixels, 30, 25, OPAQUE_WHITE);
    assert_pixel(&pixels, 6, 6, OPAQUE_WHITE);
}
