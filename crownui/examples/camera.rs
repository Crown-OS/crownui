use crownui::{
    kit::{CameraPreviewOptions, camera_preview, use_camera},
    prelude::*,
};

fn main() -> Result<(), crownui::RunError> {
    tracing_subscriber::fmt::init();
    crownui::run(
        component(|cx| {
            let camera = use_camera(cx, Default::default());
            vstack((
                camera_preview(camera).radius(16.0).w(640.0).h(360.0),
                text(bind(move |rt| format!("{:?}", camera.state.get(rt)))).text_size(13.0),
            ))
            .p(16.0)
            .gap(12.0)
            .items_center()
            .bg(rgb(0x0f172a))
            .text_color(WHITE)
        }),
        AppConfig::new("Camera").size(672, 430),
    )
}
