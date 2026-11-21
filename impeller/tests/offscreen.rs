use geometry::{CornerRadii, CornerShape, Point, Rect, WHITE, rgb};
use graphics::{
    BoundaryId, Brush, CompositeOp, Composition, CreateRenderer, DrawCommand, DrawList,
    GradientStop, LayerProps, Paint, Path, Renderer, Stroke, StrokeCap,
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

/// A 16-logical-pixel chevron with an arc under it, stroked like a Lucide icon.
fn chevron_icon() -> Path {
    Path::new()
        .move_to(Point::new(3.0, 3.0))
        .line_to(Point::new(13.0, 8.0))
        .line_to(Point::new(3.0, 13.0))
        .quad_to(Point::new(8.0, 18.0), Point::new(13.0, 13.0))
}

fn triangle() -> Path {
    Path::new()
        .move_to(Point::new(26.3, 4.6))
        .line_to(Point::new(42.2, 9.3))
        .line_to(Point::new(30.6, 20.4))
        .close()
}

fn icon_offscreen() -> Offscreen {
    Offscreen {
        width: 96,
        height: 80,
        scale: 2.0,
    }
}

#[test]
fn small_paths_have_analytic_coverage() {
    let red = Paint::solid(rgb(0xff0000));
    let mut icons = DrawList::new();
    icons.push(DrawCommand::StrokePath(
        chevron_icon(),
        Stroke::rounded(2.0),
        red.clone(),
    ));
    icons.push(DrawCommand::FillPath(triangle(), red.clone()));
    for (cap, y) in [
        (StrokeCap::Butt, 24.0),
        (StrokeCap::Square, 28.0),
        (StrokeCap::Round, 32.0),
    ] {
        let line = Path::new()
            .move_to(Point::new(4.0, y))
            .line_to(Point::new(14.0, y));
        let stroke = Stroke {
            cap,
            ..Stroke::new(2.0)
        };
        icons.push(DrawCommand::StrokePath(line, stroke, red.clone()));
    }
    let pixels = draw(
        &mut renderer(&icon_offscreen(), CornerShape::Round),
        &icons,
        &card_frame(),
    );
    assert_pixel(&pixels, 16, 11, RED);
    assert_pixel(&pixels, 12, 16, OPAQUE_WHITE);
    assert_pixel(&pixels, 66, 22, RED);
    assert_pixel(&pixels, 90, 4, OPAQUE_WHITE);
    assert_pixel(&pixels, 40, 70, OPAQUE_WHITE);
    assert_pixel(&pixels, 28, 48, OPAQUE_WHITE);
    assert_pixel(&pixels, 28, 56, RED);
    assert_pixel(&pixels, 28, 64, RED);
    for (name, from, to) in [("stroke", (4, 22), (32, 34)), ("fill", (52, 6), (84, 20))] {
        let levels = coverage_levels(&pixels, from, to);
        assert!(
            levels > MSAA_LEVELS,
            "the {name} edge has only {levels} coverage levels"
        );
    }
}

#[test]
#[ignore = "NVIDIA's EGL segfaults freeing a context Impeller drew an offscreen layer with; \
            run with __EGL_VENDOR_LIBRARY_FILENAMES pointing at Mesa and --ignored"]
fn small_gradient_paths_are_masked_smoothly() {
    let stop = |offset| GradientStop {
        offset,
        color: rgb(0xff0000),
    };
    let gradient = Paint {
        brush: Brush::LinearGradient {
            start: Point::new(0.0, 0.0),
            end: Point::new(16.0, 16.0),
            stops: [stop(0.0), stop(1.0)].into_iter().collect(),
        },
        blur: None,
    };
    let mut icons = DrawList::new();
    icons.push(DrawCommand::StrokePath(
        chevron_icon(),
        Stroke::rounded(2.0),
        gradient.clone(),
    ));
    icons.push(DrawCommand::FillPath(triangle(), gradient));
    let pixels = draw(
        &mut renderer(&icon_offscreen(), CornerShape::Round),
        &icons,
        &card_frame(),
    );
    assert_pixel(&pixels, 16, 11, RED);
    assert_pixel(&pixels, 12, 16, OPAQUE_WHITE);
    assert_pixel(&pixels, 66, 22, RED);
    assert_pixel(&pixels, 90, 4, OPAQUE_WHITE);
    let levels = coverage_levels(&pixels, (4, 22), (32, 34));
    assert!(
        levels > MSAA_LEVELS,
        "the masked stroke has only {levels} coverage levels"
    );
}

/// Left and right ends of the red run along row `y`.
fn red_span(pixels: &Pixels, y: u32) -> Option<(u32, u32)> {
    let red = |x: &u32| {
        pixels
            .pixel(*x, y)
            .is_some_and(|[r, g, ..]| r > 200 && g < 128)
    };
    let first = (0..pixels.width).find(red)?;
    let last = (0..pixels.width).rev().find(red)?;
    Some((first, last))
}

#[test]
fn a_tilted_layer_draws_in_perspective_with_its_rounded_clip() {
    let offscreen = Offscreen {
        width: 160,
        height: 160,
        scale: 1.0,
    };
    let mut card = DrawList::new();
    card.fill(
        Rect::from_xywh(0.0, 0.0, 120.0, 120.0),
        CornerRadii::uniform(20.0),
        rgb(0xff0000),
    );
    let frame = Composition {
        clear_color: WHITE,
        ops: vec![
            CompositeOp::PushLayer(LayerProps {
                transform: geometry::Transform::translate(geometry::Vector::new(20.0, 20.0)),
                tilt: Some(geometry::Tilt {
                    pitch: 40_f32.to_radians(),
                    yaw: 0.0,
                    distance: 160.0,
                    pivot: Point::new(80.0, 80.0),
                }),
                opacity: 0.99,
                clip: Some((
                    Rect::from_xywh(0.0, 0.0, 120.0, 120.0),
                    CornerRadii::uniform(20.0),
                )),
                ..LayerProps::default()
            }),
            CompositeOp::Draw(CARD),
            CompositeOp::PopLayer,
        ],
    };
    let pixels = draw(&mut renderer(&offscreen, CornerShape::Round), &card, &frame);
    if let Ok(path) = std::env::var("TILT_PNG") {
        let _ = encode_png(&pixels).map(|png| std::fs::write(path, png));
    }
    let Some((top_left, top_right)) = red_span(&pixels, 60) else {
        panic!("nothing drawn above the pivot");
    };
    let Some((bottom_left, bottom_right)) = red_span(&pixels, 100) else {
        panic!("nothing drawn below the pivot");
    };
    assert!(
        bottom_right - bottom_left > top_right - top_left + 8,
        "the near edge is not wider: top {top_left}..{top_right}, bottom {bottom_left}..{bottom_right}"
    );
    assert!(
        top_left > 20 && top_right < 140,
        "the far edge did not narrow"
    );
}

#[test]
fn a_tilted_layer_cuts_its_rounded_clip_out_of_square_content() {
    let offscreen = Offscreen {
        width: 160,
        height: 160,
        scale: 1.0,
    };
    let mut card = DrawList::new();
    card.fill(
        Rect::from_xywh(0.0, 0.0, 120.0, 120.0),
        CornerRadii::ZERO,
        rgb(0xff0000),
    );
    let offset = geometry::Vector::new(20.0, 20.0);
    let tilt = geometry::Tilt {
        pitch: 20_f32.to_radians(),
        yaw: 0.0,
        distance: 400.0,
        pivot: Point::new(80.0, 80.0),
    };
    let frame = Composition {
        clear_color: WHITE,
        ops: vec![
            CompositeOp::PushLayer(LayerProps {
                transform: geometry::Transform::translate(offset),
                tilt: Some(tilt),
                clip: Some((
                    Rect::from_xywh(0.0, 0.0, 120.0, 120.0),
                    CornerRadii::uniform(30.0),
                )),
                ..LayerProps::default()
            }),
            CompositeOp::Draw(CARD),
            CompositeOp::PopLayer,
        ],
    };
    let pixels = draw(&mut renderer(&offscreen, CornerShape::Round), &card, &frame);
    let on_screen = |x: f32, y: f32| {
        let Some(point) = tilt.apply(Point::new(x + offset.x, y + offset.y)) else {
            panic!("({x}, {y}) is behind the eye");
        };
        (point.x.round() as u32, point.y.round() as u32)
    };
    let (corner_x, corner_y) = on_screen(3.0, 3.0);
    assert_pixel(&pixels, corner_x, corner_y, OPAQUE_WHITE);
    let (inside_x, inside_y) = on_screen(30.0, 30.0);
    assert_pixel(&pixels, inside_x, inside_y, RED);
}

#[test]
fn a_transform_moved_as_an_image_lands_where_the_drawn_one_does_inside_a_tilt() {
    let offscreen = Offscreen {
        width: 200,
        height: 200,
        scale: 1.25,
    };
    let mut card = DrawList::new();
    card.fill(
        Rect::from_xywh(0.0, 0.0, 100.0, 100.0),
        CornerRadii::ZERO,
        rgb(0xff0000),
    );
    let frame = |transform_as_image| Composition {
        clear_color: WHITE,
        ops: vec![
            CompositeOp::PushLayer(LayerProps {
                transform: geometry::Transform::translate(geometry::Vector::new(20.0, 20.0)),
                tilt: Some(geometry::Tilt {
                    pitch: 12_f32.to_radians(),
                    yaw: 0.0,
                    distance: 300.0,
                    pivot: Point::new(80.0, 80.0),
                }),
                clip: Some((
                    Rect::from_xywh(0.0, 0.0, 120.0, 120.0),
                    CornerRadii::uniform(24.0),
                )),
                ..LayerProps::default()
            }),
            CompositeOp::PushLayer(LayerProps {
                transform: geometry::Transform::scale(0.8, 0.9),
                transform_as_image,
                opacity: 0.8,
                ..LayerProps::default()
            }),
            CompositeOp::Draw(CARD),
            CompositeOp::PopLayer,
            CompositeOp::PopLayer,
        ],
    };
    let mut renderer = renderer(&offscreen, CornerShape::Round);
    let drawn = draw(&mut renderer, &card, &frame(false));
    let moved = draw(&mut renderer, &card, &frame(true));
    for y in (40..140).step_by(10) {
        let (Some(drawn_span), Some(moved_span)) = (red_span(&drawn, y), red_span(&moved, y))
        else {
            panic!(
                "row {y}: drawn {:?}, as image {:?}",
                red_span(&drawn, y),
                red_span(&moved, y)
            );
        };
        assert!(
            drawn_span.0.abs_diff(moved_span.0) <= 2 && drawn_span.1.abs_diff(moved_span.1) <= 2,
            "row {y}: drawn {drawn_span:?}, as image {moved_span:?}"
        );
    }
}
