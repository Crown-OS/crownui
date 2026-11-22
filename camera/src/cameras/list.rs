use crate::CameraInfo;

/// The cameras the registry announced, by global id, in the order they appeared.
#[derive(Debug, Default)]
pub(crate) struct CameraList {
    cameras: Vec<(u32, CameraInfo)>,
}

impl CameraList {
    /// Adds or replaces the camera behind `global`; returns whether the list changed.
    pub(crate) fn add(&mut self, global: u32, camera: CameraInfo) -> bool {
        match self.cameras.iter_mut().find(|(known, _)| *known == global) {
            Some((_, existing)) if *existing == camera => false,
            Some((_, existing)) => {
                *existing = camera;
                true
            }
            None => {
                self.cameras.push((global, camera));
                true
            }
        }
    }

    /// Forgets `global`; returns whether it was a camera.
    pub(crate) fn remove(&mut self, global: u32) -> bool {
        let before = self.cameras.len();
        self.cameras.retain(|(known, _)| *known != global);
        self.cameras.len() != before
    }

    pub(crate) fn cameras(&self) -> Vec<CameraInfo> {
        self.cameras
            .iter()
            .map(|(_, camera)| camera.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CameraId;

    fn camera(name: &str) -> CameraInfo {
        CameraInfo {
            id: CameraId::new(name),
            description: format!("{name} camera"),
        }
    }

    #[test]
    fn cameras_come_and_go_in_order() {
        let mut list = CameraList::default();
        assert!(list.add(40, camera("front")));
        assert!(list.add(57, camera("usb")));
        assert!(!list.add(57, camera("usb")));
        assert_eq!(list.cameras(), [camera("front"), camera("usb")]);
        assert!(list.remove(40));
        assert!(!list.remove(40));
        assert_eq!(list.cameras(), [camera("usb")]);
    }

    #[test]
    fn a_reannounced_global_replaces_its_camera() {
        let mut list = CameraList::default();
        list.add(40, camera("front"));
        assert!(list.add(40, camera("renamed")));
        assert_eq!(list.cameras(), [camera("renamed")]);
    }
}
