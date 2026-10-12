use crownos_protocols::surface_visibility::v1::client::{
    crownos_surface_visibility_manager_v1::CrownosSurfaceVisibilityManagerV1,
    crownos_surface_visibility_v1::{self, CrownosSurfaceVisibilityV1, State},
};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{Connection, Proxy, QueueHandle, WEnum, globals::GlobalList},
};

use super::CrownEvent;
use crate::{
    WindowId,
    wayland::{
        Window,
        error::Error,
        event_loop::{QueuedEvent, WaylandPlatform},
        util::no_events::NoEvents,
    },
};

const NORMAL_SCALE: f64 = 1.0;

/// Whether the compositor draws a window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SurfaceVisibility {
    #[default]
    Visible,
    /// On screen but fully covered, e.g. by a fullscreen window.
    Occluded,
    /// Not drawn anywhere: another workspace, unmapped, or the session is locked.
    Hidden,
}

impl From<&SurfaceVisibility> for crate::Visibility {
    fn from(visibility: &SurfaceVisibility) -> Self {
        match visibility {
            SurfaceVisibility::Visible => Self::Visible,
            SurfaceVisibility::Occluded => Self::Occluded,
            SurfaceVisibility::Hidden => Self::Hidden,
        }
    }
}

impl From<WEnum<State>> for SurfaceVisibility {
    fn from(state: WEnum<State>) -> Self {
        match state {
            WEnum::Value(State::Occluded) => Self::Occluded,
            WEnum::Value(State::Hidden) => Self::Hidden,
            _ => Self::Visible,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct VisibilityReport {
    state: SurfaceVisibility,
    thumbnail_scale: f64,
}

impl Default for VisibilityReport {
    fn default() -> Self {
        Self {
            state: SurfaceVisibility::Visible,
            thumbnail_scale: NORMAL_SCALE,
        }
    }
}

pub(crate) struct SurfaceVisibilityGlobal {
    manager: CrownosSurfaceVisibilityManagerV1,
    reports: Vec<(CrownosSurfaceVisibilityV1, VisibilityReport)>,
}

impl SurfaceVisibilityGlobal {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<WaylandPlatform>) -> Option<Self> {
        Some(Self {
            manager: globals.bind(qh, 1..=1, NoEvents).ok()?,
            reports: Vec::new(),
        })
    }

    fn report_mut(&mut self, observer: &CrownosSurfaceVisibilityV1) -> &mut VisibilityReport {
        self.reports.retain(|(proxy, _)| proxy.is_alive());
        let index = match self.reports.iter().position(|(proxy, _)| proxy == observer) {
            Some(index) => index,
            None => {
                self.reports
                    .push((observer.clone(), VisibilityReport::default()));
                self.reports.len() - 1
            }
        };
        &mut self.reports[index].1
    }
}

/// Reports a window's [`SurfaceVisibility`] as
/// [`CrownEvent::SurfaceVisibility`]; dropping it stops the reports.
#[derive(Debug)]
pub struct SurfaceVisibilityObserver {
    proxy: CrownosSurfaceVisibilityV1,
}

impl WaylandPlatform {
    /// The compositor sends the current visibility right away, then every change.
    pub fn observe_visibility(&self, window: &Window) -> Result<SurfaceVisibilityObserver, Error> {
        let visibility = self.crown.visibility.as_ref().ok_or(Error::MissingGlobal(
            "crownos_surface_visibility_manager_v1",
        ))?;
        let proxy = visibility.manager.get_surface_visibility(
            window.wl_surface(),
            &self.queue_handle,
            SurfaceVisibilityData {
                window: window.id(),
            },
        );
        Ok(SurfaceVisibilityObserver { proxy })
    }
}

impl Drop for SurfaceVisibilityObserver {
    fn drop(&mut self) {
        self.proxy.destroy();
    }
}

pub(crate) struct SurfaceVisibilityData {
    window: WindowId,
}

impl Dispatch2<CrownosSurfaceVisibilityV1, WaylandPlatform> for SurfaceVisibilityData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        observer: &CrownosSurfaceVisibilityV1,
        event: crownos_surface_visibility_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let Some(visibility) = state.crown.visibility.as_mut() else {
            return;
        };
        let report = visibility.report_mut(observer);
        match event {
            crownos_surface_visibility_v1::Event::State { state: visible } => {
                report.state = visible.into();
            }
            crownos_surface_visibility_v1::Event::ThumbnailScale { scale } => {
                report.thumbnail_scale = scale;
            }
            crownos_surface_visibility_v1::Event::Done => {
                let report = *report;
                state
                    .pending_events
                    .push(QueuedEvent::Crown(CrownEvent::SurfaceVisibility {
                        window: self.window,
                        state: report.state,
                        thumbnail_scale: report.thumbnail_scale,
                    }));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_states_map_to_their_visibility() {
        assert_eq!(
            SurfaceVisibility::from(WEnum::Value(State::Visible)),
            SurfaceVisibility::Visible
        );
        assert_eq!(
            SurfaceVisibility::from(WEnum::Value(State::Occluded)),
            SurfaceVisibility::Occluded
        );
        assert_eq!(
            SurfaceVisibility::from(WEnum::Value(State::Hidden)),
            SurfaceVisibility::Hidden
        );
    }

    #[test]
    fn an_unknown_state_from_a_newer_compositor_counts_as_visible() {
        assert_eq!(
            SurfaceVisibility::from(WEnum::<State>::Unknown(7)),
            SurfaceVisibility::Visible
        );
    }
}
