use windowing::{AppWindows, WindowEvent};

use graphics::GraphicsBackend;

use crate::TestApp;

/// What the compositor tells a launcher about the apps behind its icons.
impl<G: GraphicsBackend> TestApp<G> {
    /// Reports that an app has `windows.count` windows, one of them focused or not, as the
    /// compositor does on every change (see `Cx::on_app_windows`).
    pub fn app_windows(&mut self, windows: AppWindows) -> &mut Self {
        self.deliver(WindowEvent::AppWindows(windows))
    }

    /// The compositor starts drawing the icon of `app_id` itself (see `Cx::on_app_icon_lifted`).
    pub fn lift_app_icon(&mut self, app_id: &str) -> &mut Self {
        self.carry_app_icon(app_id, true)
    }

    /// The compositor stops drawing the icon of `app_id`, handing it back.
    pub fn land_app_icon(&mut self, app_id: &str) -> &mut Self {
        self.carry_app_icon(app_id, false)
    }

    /// The compositor asks the launcher to start `app_id` (see `Cx::on_launch_app`).
    pub fn launch_app(&mut self, app_id: &str) -> &mut Self {
        self.deliver(WindowEvent::LaunchApp(app_id.to_owned()))
    }

    fn carry_app_icon(&mut self, app_id: &str, lifted: bool) -> &mut Self {
        self.deliver(WindowEvent::AppIconLifted {
            app_id: app_id.to_owned(),
            lifted,
        })
    }
}
