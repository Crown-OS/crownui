//! Renders into a GBM buffer on a real GPU and reads it back through the CPU mapping.

use std::{fs::OpenOptions, os::fd::AsFd, path::Path};

use gbm::{BufferObjectFlags, Device, Format};
use geometry::{CornerRadii, Rect, WHITE, rgb};
use graphics::{
    BoundaryId, CompositeOp, Composition, CreateRenderer, DrawList, LayerProps, Renderer,
};
use impeller::{DmabufImage, DmabufPlane, ImpellerBackend, RenderNode};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 96;
const CARD: BoundaryId = BoundaryId(1);
const RENDER_NODE: &str = "/dev/dri/renderD128";

#[test]
fn frames_land_in_the_imported_dmabuf() {
    let node = Path::new(RENDER_NODE);
    let Ok(file) = OpenOptions::new().read(true).write(true).open(node) else {
        eprintln!("no {RENDER_NODE}; skipping");
        return;
    };
    let device = Device::new(file).expect("gbm device");
    let buffer = device
        .create_buffer_object::<()>(
            WIDTH,
            HEIGHT,
            Format::Abgr8888,
            BufferObjectFlags::RENDERING | BufferObjectFlags::LINEAR,
        )
        .expect("linear ABGR8888 buffer");
    let fd = buffer.fd_for_plane(0).expect("dma-buf fd");

    let mut renderer = ImpellerBackend::create_renderer(&RenderNode {
        path: node,
        width: WIDTH,
        height: HEIGHT,
        scale: 1.0,
    })
    .expect("render-node renderer");
    renderer
        .import_dmabuf(
            7,
            &DmabufImage {
                width: WIDTH,
                height: HEIGHT,
                fourcc: Format::Abgr8888 as u32,
                modifier: u64::from(buffer.modifier()),
                plane: DmabufPlane {
                    fd: fd.as_fd(),
                    offset: buffer.offset(0),
                    stride: buffer.stride(),
                },
            },
        )
        .expect("import");
    assert!(renderer.select_dmabuf(7));
    assert!(!renderer.select_dmabuf(8), "unknown buffers are refused");
    assert!(renderer.select_dmabuf(7));

    let mut card = DrawList::new();
    card.fill(
        Rect::from_xywh(0.0, 0.0, 64.0, 24.0),
        CornerRadii::uniform(0.0),
        rgb(0xff0000),
    );
    renderer.record_boundary(CARD, &card);
    renderer
        .render(&Composition {
            clear_color: WHITE,
            ops: vec![
                CompositeOp::PushLayer(LayerProps::default()),
                CompositeOp::Draw(CARD),
                CompositeOp::PopLayer,
            ],
        })
        .expect("render");

    let stride = buffer.stride() as usize;
    let rows = buffer
        .map(0, 0, WIDTH, HEIGHT, |mapped| mapped.buffer().to_vec())
        .expect("map the buffer");
    let pixel = |x: usize, y: usize| -> [u8; 4] {
        let at = y * stride + x * 4;
        [rows[at], rows[at + 1], rows[at + 2], rows[at + 3]]
    };
    // GL writes bottom-up, so the card in the top-left lands in the last rows; the host
    // flips when it composites.
    let flipped = |y: usize| HEIGHT as usize - 1 - y;
    assert_eq!(pixel(10, flipped(10)), [255, 0, 0, 255], "the card");
    assert_eq!(
        pixel(100, flipped(10)),
        [255, 255, 255, 255],
        "beside the card"
    );
    assert_eq!(
        pixel(10, flipped(80)),
        [255, 255, 255, 255],
        "below the card"
    );
}
