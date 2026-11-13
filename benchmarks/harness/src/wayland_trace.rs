use std::collections::{HashMap, HashSet};

const OUTPUT_MODE_CURRENT: u32 = 1;
/// `crownos_surface_visibility_v1.state`: `visible`.
const SURFACE_VISIBLE: u32 = 0;

/// What one `WAYLAND_DEBUG=client` line means for the measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceEvent {
    /// A surface tree presented new content: a commit that applies a newly attached buffer.
    Frame,
    /// The current mode of an output, in millihertz.
    Refresh(u32),
    /// The compositor let the client draw again; the argument is its clock in milliseconds.
    FrameCallback(u32),
    /// crownpositor says nothing shows the window (hidden, covered, or the screen is locked).
    NotShown,
}

#[derive(Debug, Default)]
struct SurfaceState {
    parent: Option<u32>,
    desynchronized: bool,
    has_new_buffer: bool,
}

/// Follows surfaces, subsurfaces and buffer attachments through a client's protocol trace and
/// reports a frame each time new pixels reach the compositor. Synchronized subsurfaces hand
/// their pending buffer to the parent, so a parent and its children committing together count
/// once.
#[derive(Debug, Default)]
pub struct FrameTracker {
    surfaces: HashMap<u32, SurfaceState>,
    subsurface_children: HashMap<u32, u32>,
    frame_callbacks: HashSet<u32>,
}

impl FrameTracker {
    pub fn observe(&mut self, line: &str) -> Option<TraceEvent> {
        let message = Message::parse(line)?;
        match (message.interface, message.method) {
            ("wl_output", "mode") => current_refresh(message.arguments),
            ("crownos_surface_visibility_v1", "state") if !message.request => {
                let state: u32 = message.arguments.parse().ok()?;
                (state != SURFACE_VISIBLE).then_some(TraceEvent::NotShown)
            }
            ("wl_subcompositor", "get_subsurface") if message.request => {
                self.add_subsurface(message.arguments);
                None
            }
            ("wl_subsurface", "set_desync" | "set_sync") if message.request => {
                let child = self.subsurface_children.get(&message.object)?;
                self.surfaces.entry(*child).or_default().desynchronized =
                    message.method == "set_desync";
                None
            }
            ("wl_subsurface", "destroy") if message.request => {
                let child = self.subsurface_children.remove(&message.object)?;
                self.surfaces.entry(child).or_default().parent = None;
                None
            }
            ("wl_surface", "attach") if message.request => {
                self.surfaces
                    .entry(message.object)
                    .or_default()
                    .has_new_buffer = !message.arguments.starts_with("nil");
                None
            }
            ("wl_surface", "commit") if message.request => self.commit(message.object),
            ("wl_surface", "frame") if message.request => {
                self.frame_callbacks.insert(object_id(message.arguments)?);
                None
            }
            ("wl_callback", "done") if !message.request => {
                self.frame_callbacks.remove(&message.object).then(|| {
                    message
                        .arguments
                        .parse()
                        .ok()
                        .map(TraceEvent::FrameCallback)
                })?
            }
            ("wl_surface", "destroy") if message.request => {
                self.surfaces.remove(&message.object);
                None
            }
            _ => None,
        }
    }

    fn add_subsurface(&mut self, arguments: &str) {
        let mut ids = arguments.split(", ").filter_map(object_id);
        let (Some(subsurface), Some(child), Some(parent)) = (ids.next(), ids.next(), ids.next())
        else {
            return;
        };
        self.subsurface_children.insert(subsurface, child);
        self.surfaces.entry(child).or_default().parent = Some(parent);
    }

    fn commit(&mut self, surface: u32) -> Option<TraceEvent> {
        let state = self.surfaces.entry(surface).or_default();
        let has_new_buffer = std::mem::take(&mut state.has_new_buffer);
        match (state.parent, state.desynchronized) {
            (Some(parent), false) => {
                self.surfaces.entry(parent).or_default().has_new_buffer |= has_new_buffer;
                None
            }
            _ => has_new_buffer.then_some(TraceEvent::Frame),
        }
    }
}

/// `[1402478.391] {Default Queue}  -> wl_surface#36.attach(wl_buffer#51, 0, 0)` for a request,
/// the same without `->` for an event.
struct Message<'a> {
    request: bool,
    interface: &'a str,
    object: u32,
    method: &'a str,
    arguments: &'a str,
}

impl<'a> Message<'a> {
    fn parse(line: &'a str) -> Option<Self> {
        let rest = line.strip_prefix('[')?;
        let mut rest = rest[rest.find(']')? + 1..].trim_start();
        if let Some(queue) = rest.strip_prefix('{') {
            rest = queue[queue.find('}')? + 1..].trim_start();
        }
        let (request, rest) = match rest.strip_prefix("-> ") {
            Some(request) => (true, request.trim_end()),
            None => (false, rest.trim_end()),
        };
        let (target, call) = rest.split_once('.')?;
        let (interface, object) = target.split_once('#')?;
        let (method, arguments) = call.split_once('(')?;
        Some(Self {
            request,
            interface,
            object: object.parse().ok()?,
            method,
            arguments: arguments.strip_suffix(')')?,
        })
    }
}

fn object_id(argument: &str) -> Option<u32> {
    argument.rsplit_once('#')?.1.parse().ok()
}

fn current_refresh(arguments: &str) -> Option<TraceEvent> {
    let mut fields = arguments.split(", ").map(str::parse::<u32>);
    let flags = fields.next()?.ok()?;
    let refresh = fields.nth(2)?.ok()?;
    (flags & OUTPUT_MODE_CURRENT != 0).then_some(TraceEvent::Refresh(refresh))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(lines: &[&str]) -> usize {
        let mut tracker = FrameTracker::default();
        lines
            .iter()
            .filter(|line| tracker.observe(line) == Some(TraceEvent::Frame))
            .count()
    }

    #[test]
    fn a_commit_without_a_buffer_is_not_a_frame() {
        assert_eq!(
            frames(&[
                "[1402282.133]  -> wl_surface#36.commit()",
                "[1402478.391]  -> wl_surface#36.attach(wl_buffer#51, 0, 0)",
                "[1402478.427]  -> wl_surface#36.commit()",
                "[1402479.000]  -> wl_surface#36.commit()",
            ]),
            1
        );
    }

    #[test]
    fn attaching_nil_unmaps_rather_than_presents() {
        assert_eq!(
            frames(&[
                "[1.000]  -> wl_surface#3.attach(nil, 0, 0)",
                "[1.001]  -> wl_surface#3.commit()",
            ]),
            0
        );
    }

    #[test]
    fn a_synchronized_subsurface_presents_with_its_parent() {
        assert_eq!(
            frames(&[
                "[1.000]  -> wl_subcompositor#9.get_subsurface(new id wl_subsurface#20, wl_surface#12, wl_surface#10)",
                "[1.001]  -> wl_surface#12.attach(wl_buffer#30, 0, 0)",
                "[1.002]  -> wl_surface#12.commit()",
                "[1.003]  -> wl_surface#10.commit()",
                "[1.004]  -> wl_surface#12.attach(wl_buffer#31, 0, 0)",
                "[1.005]  -> wl_surface#12.commit()",
                "[1.006]  -> wl_surface#10.attach(wl_buffer#32, 0, 0)",
                "[1.007]  -> wl_surface#10.commit()",
            ]),
            2
        );
    }

    #[test]
    fn a_desynchronized_subsurface_presents_on_its_own() {
        assert_eq!(
            frames(&[
                "[1.000]  -> wl_subcompositor#9.get_subsurface(new id wl_subsurface#20, wl_surface#12, wl_surface#10)",
                "[1.001]  -> wl_subsurface#20.set_desync()",
                "[1.002]  -> wl_surface#12.attach(wl_buffer#30, 0, 0)",
                "[1.003]  -> wl_surface#12.commit()",
            ]),
            1
        );
    }

    #[test]
    fn queue_names_and_events_parse() {
        let mut tracker = FrameTracker::default();
        assert_eq!(
            tracker.observe("[  12.500] {Default Queue} wl_output#7.mode(1, 1920, 1200, 120000)"),
            Some(TraceEvent::Refresh(120_000))
        );
        assert_eq!(
            tracker.observe("[  12.600] {Default Queue} wl_output#7.mode(0, 1280, 800, 60000)"),
            None
        );
        tracker.observe("[  13.000] {Default Queue}  -> wl_surface#5.attach(wl_buffer#8, 0, 0)");
        assert_eq!(
            tracker.observe("[  13.001] {Default Queue}  -> wl_surface#5.commit()"),
            Some(TraceEvent::Frame)
        );
    }

    #[test]
    fn only_frame_callbacks_report_the_compositor_clock() {
        let mut tracker = FrameTracker::default();
        tracker.observe("[1.000]  -> wl_display#1.sync(new id wl_callback#3)");
        assert_eq!(tracker.observe("[1.001] wl_callback#3.done(118886)"), None);
        tracker.observe("[1.002]  -> wl_surface#5.frame(new id wl_callback#9)");
        assert_eq!(
            tracker.observe("[1.003] wl_callback#9.done(9446489)"),
            Some(TraceEvent::FrameCallback(9_446_489))
        );
    }

    #[test]
    fn a_hidden_surface_is_reported() {
        let mut tracker = FrameTracker::default();
        assert_eq!(
            tracker.observe("[1.000] crownos_surface_visibility_v1#51.state(2)"),
            Some(TraceEvent::NotShown)
        );
        assert_eq!(
            tracker.observe("[1.001] crownos_surface_visibility_v1#51.state(0)"),
            None
        );
    }

    #[test]
    fn non_wayland_lines_are_ignored() {
        let mut tracker = FrameTracker::default();
        assert_eq!(
            tracker
                .observe("[39578:1107/014916.726670:ERROR:node_bindings.cc:492] Most NODE_OPTIONs"),
            None
        );
    }
}
