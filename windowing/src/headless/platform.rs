use std::{collections::BTreeMap, convert::Infallible, time::Duration};

use super::{HeadlessSurface, HeadlessWindow};
use crate::{
    ApplicationHandler, Capabilities, Clock, ControlFlow, ManualClock, Platform, WindowAttributes,
    WindowEvent, WindowId, dpi::ScaleFactor,
};

/// The headless platform; drive it with [`start`](Self::start), [`inject`](Self::inject),
/// [`advance`](Self::advance) and [`run_until_idle`](Self::run_until_idle).
#[derive(Debug)]
pub struct HeadlessPlatform {
    windows: BTreeMap<WindowId, HeadlessWindow>,
    pending: Vec<(WindowId, WindowEvent)>,
    next_window: u64,
    clock: ManualClock,
    capabilities: Capabilities,
    control_flow: ControlFlow,
    exit_requested: bool,
    clipboard: String,
    focused: Option<WindowId>,
}

impl Default for HeadlessPlatform {
    fn default() -> Self {
        Self::new(Capabilities {
            materials: true,
            fractional_scale: true,
            touchpad_gestures: true,
            text_input: true,
            popups: true,
            presentation_time: true,
            visibility_hints: false,
            surface_animation: false,
            layer_shell: true,
            idle_inhibit: true,
            app_icons: false,
        })
    }
}

impl HeadlessPlatform {
    /// A platform that claims `capabilities`, so tests can exercise fallbacks.
    pub fn new(capabilities: Capabilities) -> Self {
        Self {
            windows: BTreeMap::new(),
            pending: Vec::new(),
            next_window: 1,
            clock: ManualClock::default(),
            capabilities,
            control_flow: ControlFlow::Wait,
            exit_requested: false,
            clipboard: String::new(),
            focused: None,
        }
    }

    /// The platform's clock.
    pub fn clock(&self) -> &ManualClock {
        &self.clock
    }

    /// Current time on the platform's clock.
    pub fn now(&self) -> Duration {
        self.clock.now()
    }

    /// Moves the clock forward.
    pub fn advance(&mut self, elapsed: Duration) {
        self.clock.advance(elapsed);
    }

    /// Queues `event` for `window`, as if a user caused it.
    pub fn inject(&mut self, window: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Focused(true) => self.focused = Some(window),
            WindowEvent::Focused(false) if self.focused == Some(window) => self.focused = None,
            _ => {}
        }
        self.pending.push((window, event));
    }

    /// Changes a window's scale and queues the events a real platform would send.
    pub fn set_scale_factor(&mut self, window: WindowId, scale: ScaleFactor) {
        let Some(state) = self.windows.get_mut(&window) else {
            return;
        };
        state.scale_factor = scale;
        let size = state.logical_size.to_physical(scale);
        self.pending.extend([
            (window, WindowEvent::ScaleFactorChanged(scale)),
            (window, WindowEvent::Resized(size)),
        ]);
    }

    /// Whether the app asked to exit.
    pub fn exiting(&self) -> bool {
        self.exit_requested
    }

    /// Calls [`ApplicationHandler::resumed`].
    pub fn start<A: ApplicationHandler<Self>>(&mut self, app: &mut A) {
        app.resumed(self);
        self.run_until_idle(app);
    }

    /// Delivers queued events and redraw requests until nothing is left.
    pub fn run_until_idle<A: ApplicationHandler<Self>>(&mut self, app: &mut A) {
        loop {
            let redraws = self.windows.values_mut().filter_map(|window| {
                std::mem::take(&mut window.redraw_requested).then_some(window.id)
            });
            let redraws: Vec<_> = redraws.collect();
            self.pending.extend(
                redraws
                    .into_iter()
                    .map(|window| (window, WindowEvent::RedrawRequested)),
            );
            if self.pending.is_empty() || self.exit_requested {
                app.about_to_wait(self);
                return;
            }
            for (window, event) in std::mem::take(&mut self.pending) {
                app.window_event(self, window, event);
            }
        }
    }
}

impl Platform for HeadlessPlatform {
    type Window = HeadlessWindow;
    type Surface = HeadlessSurface;
    type Event = Infallible;
    type Error = Infallible;

    fn create_window(&mut self, attributes: WindowAttributes) -> Result<WindowId, Infallible> {
        let id = WindowId::from_raw(self.next_window);
        self.next_window += 1;
        let window = HeadlessWindow {
            id,
            title: attributes.title,
            logical_size: attributes.inner_size,
            scale_factor: ScaleFactor::IDENTITY,
            cursor: Default::default(),
            material: attributes.material,
            text_input: None,
            idle_inhibited: false,
            layer: attributes.layer,
            redraw_requested: true,
        };
        self.pending.push((
            id,
            WindowEvent::Resized(attributes.inner_size.to_physical(ScaleFactor::IDENTITY)),
        ));
        self.windows.insert(id, window);
        Ok(id)
    }

    fn window(&self, id: WindowId) -> Option<&HeadlessWindow> {
        self.windows.get(&id)
    }

    fn window_mut(&mut self, id: WindowId) -> Option<&mut HeadlessWindow> {
        self.windows.get_mut(&id)
    }

    fn close_window(&mut self, id: WindowId) {
        self.windows.remove(&id);
    }

    fn surface(&self, id: WindowId) -> Option<HeadlessSurface> {
        self.windows.contains_key(&id).then(|| HeadlessSurface::new(id))
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    fn set_control_flow(&mut self, control_flow: ControlFlow) {
        self.control_flow = control_flow;
    }

    fn exit(&mut self) {
        self.exit_requested = true;
    }

    fn waker(&self) -> crate::Waker {
        Box::new(|| {})
    }

    fn set_clipboard_text(&mut self, text: String) {
        self.clipboard = text;
    }

    fn request_clipboard_text(&mut self) {
        let target = self.focused.or_else(|| self.windows.keys().next().copied());
        if let Some(window) = target {
            let text = self.clipboard.clone();
            self.pending.push((window, WindowEvent::ClipboardText(text)));
        }
    }

    /// No other app can take the data, so every drag ends unclaimed.
    fn start_drag(&mut self, window: WindowId, _: crate::DragSource) {
        self.pending.push((
            window,
            WindowEvent::DragEnded(crate::DragOutcome::Cancelled),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WindowControl, dpi::LogicalSize};

    #[derive(Default)]
    struct Recorder {
        events: Vec<WindowEvent>,
        window: Option<WindowId>,
    }

    impl ApplicationHandler<HeadlessPlatform> for Recorder {
        fn resumed(&mut self, platform: &mut HeadlessPlatform) {
            let attributes = WindowAttributes::default().with_inner_size(LogicalSize::new(100, 50));
            self.window = platform.create_window(attributes).ok();
        }

        fn window_event(&mut self, _: &mut HeadlessPlatform, _: WindowId, event: WindowEvent) {
            self.events.push(event);
        }
    }

    #[test]
    fn a_new_window_is_sized_then_drawn() {
        let mut platform = HeadlessPlatform::default();
        let mut app = Recorder::default();
        platform.start(&mut app);
        assert_eq!(
            app.events,
            [
                WindowEvent::Resized(crate::dpi::PhysicalSize {
                    width: 100,
                    height: 50
                }),
                WindowEvent::RedrawRequested
            ]
        );
    }

    #[test]
    fn clipboard_text_round_trips_to_the_focused_window() {
        let mut platform = HeadlessPlatform::default();
        let mut app = Recorder::default();
        platform.start(&mut app);
        let Some(window) = app.window else {
            panic!("window created in resumed");
        };
        platform.inject(window, WindowEvent::Focused(true));
        platform.set_clipboard_text("copied".to_owned());
        platform.request_clipboard_text();
        platform.run_until_idle(&mut app);
        assert_eq!(
            app.events.last(),
            Some(&WindowEvent::ClipboardText("copied".to_owned()))
        );
    }

    #[test]
    fn scale_changes_resize_the_drawable() {
        let mut platform = HeadlessPlatform::default();
        let mut app = Recorder::default();
        platform.start(&mut app);
        let Some(window) = app.window else {
            panic!("window created in resumed");
        };
        app.events.clear();
        platform.set_scale_factor(window, ScaleFactor::from_integer(2));
        platform.run_until_idle(&mut app);
        assert_eq!(
            platform.window(window).map(WindowControl::inner_size),
            Some(crate::dpi::PhysicalSize {
                width: 200,
                height: 100
            })
        );
        assert_eq!(app.events.len(), 2);
    }
}
