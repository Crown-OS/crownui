use geometry::{CornerRadii, Point, Rect, WHITE, rgb};
use graphics::{
    BoundaryId, CompositeOp, Composition, CreateRenderer, DrawCommand, DrawList, LayerProps, Paint,
    Path, Renderer,
};
use impeller::{ImpellerBackend, Offscreen, Pixels, RenderApi, encode_png};

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
