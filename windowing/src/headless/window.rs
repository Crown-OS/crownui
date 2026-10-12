use crate::{
    CursorIcon, LayerOptions, Material, TextInputState, WindowControl, WindowId,
    dpi::{LogicalSize, PhysicalSize, ScaleFactor},
};

/// A window that only exists in memory; its state is inspectable by tests.
#[derive(Debug, Clone)]
pub struct HeadlessWindow {
    pub(crate) id: WindowId,
    /// Current title.
    pub title: String,
    /// Content size.
    pub logical_size: LogicalSize,
    /// Scale factor.
    pub scale_factor: ScaleFactor,
    /// Cursor last requested.
    pub cursor: CursorIcon,
    /// Material last requested.
    pub material: Option<Material>,
    /// Input-method state last requested; `None` while text input is off.
    pub text_input: Option<TextInputState>,
    /// Whether the window last asked to keep the system awake.
    pub idle_inhibited: bool,
    /// The layer-surface options it was created with, if it is one.
    pub layer: Option<LayerOptions>,
    pub(crate) redraw_requested: bool,
}

impl WindowControl for HeadlessWindow {
    fn id(&self) -> WindowId {
        self.id
    }

    fn logical_size(&self) -> LogicalSize {
        self.logical_size
    }

    fn inner_size(&self) -> PhysicalSize {
        self.logical_size.to_physical(self.scale_factor)
    }

    fn scale_factor(&self) -> ScaleFactor {
        self.scale_factor
    }

    fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    fn set_title(&mut self, title: &str) {
        title.clone_into(&mut self.title);
    }

    fn set_cursor(&mut self, icon: CursorIcon) {
        self.cursor = icon;
    }

    fn set_material(&mut self, material: Option<&Material>) {
        self.material = material.copied();
    }

    fn begin_move(&mut self) {}

    fn set_text_input(&mut self, state: Option<TextInputState>) {
        self.text_input = state;
    }

    fn set_idle_inhibited(&mut self, inhibited: bool) {
        self.idle_inhibited = inhibited;
    }
}
