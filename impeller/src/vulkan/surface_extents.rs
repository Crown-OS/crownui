use std::sync::atomic::{AtomicU64, Ordering};

use ash::vk::{self, Handle};

const SLOT_COUNT: usize = 32;
const UNKNOWN: u64 = 0;

struct Slot {
    surface: AtomicU64,
    wanted_extent: AtomicU64,
    swapchain: AtomicU64,
    swapchain_extent: AtomicU64,
}

impl Slot {
    const fn empty() -> Self {
        Self {
            surface: AtomicU64::new(UNKNOWN),
            wanted_extent: AtomicU64::new(UNKNOWN),
            swapchain: AtomicU64::new(UNKNOWN),
            swapchain_extent: AtomicU64::new(UNKNOWN),
        }
    }
}

static SLOTS: [Slot; SLOT_COUNT] = [const { Slot::empty() }; SLOT_COUNT];

fn pack(extent: vk::Extent2D) -> u64 {
    (u64::from(extent.width) << 32) | u64::from(extent.height)
}

fn unpack(packed: u64) -> Option<vk::Extent2D> {
    (packed != UNKNOWN).then_some(vk::Extent2D {
        width: (packed >> 32) as u32,
        height: packed as u32,
    })
}

fn slot_where(field: impl Fn(&Slot) -> &AtomicU64, raw_handle: u64) -> Option<&'static Slot> {
    SLOTS
        .iter()
        .find(|slot| raw_handle != UNKNOWN && field(slot).load(Ordering::Acquire) == raw_handle)
}

/// The extent a `VkSurfaceKHR` should be presented at, read by the loader hooks; the slot is
/// released on drop.
pub(crate) struct SurfaceExtent {
    slot: &'static Slot,
}

impl SurfaceExtent {
    /// Starts tracking `surface`; `None` when every slot is taken.
    pub(crate) fn claim(surface: vk::SurfaceKHR) -> Option<Self> {
        let raw_surface = surface.as_raw();
        SLOTS
            .iter()
            .find(|slot| {
                slot.surface
                    .compare_exchange(UNKNOWN, raw_surface, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
            })
            .map(|slot| Self { slot })
    }

    pub(crate) fn capacity() -> usize {
        SLOT_COUNT
    }

    /// Makes the next acquire report the swapchain out of date unless it already has this size.
    pub(crate) fn set_wanted(&self, width: u32, height: u32) {
        let extent = vk::Extent2D {
            width: width.max(1),
            height: height.max(1),
        };
        self.slot
            .wanted_extent
            .store(pack(extent), Ordering::Release);
    }

    /// Size of the swapchain Impeller last created for this surface.
    pub(crate) fn swapchain_extent(&self) -> Option<(u32, u32)> {
        unpack(self.slot.swapchain_extent.load(Ordering::Acquire))
            .map(|extent| (extent.width, extent.height))
    }
}

impl Drop for SurfaceExtent {
    fn drop(&mut self) {
        self.slot.wanted_extent.store(UNKNOWN, Ordering::Relaxed);
        self.slot.swapchain.store(UNKNOWN, Ordering::Relaxed);
        self.slot.swapchain_extent.store(UNKNOWN, Ordering::Relaxed);
        self.slot.surface.store(UNKNOWN, Ordering::Release);
    }
}

pub(super) fn wanted_extent(surface: vk::SurfaceKHR) -> Option<vk::Extent2D> {
    slot_where(|slot| &slot.surface, surface.as_raw())
        .and_then(|slot| unpack(slot.wanted_extent.load(Ordering::Acquire)))
}

pub(super) fn record_swapchain(
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    extent: vk::Extent2D,
) {
    if let Some(slot) = slot_where(|slot| &slot.surface, surface.as_raw()) {
        slot.swapchain_extent.store(pack(extent), Ordering::Release);
        slot.swapchain.store(swapchain.as_raw(), Ordering::Release);
    }
}

pub(super) fn is_stale(swapchain: vk::SwapchainKHR) -> bool {
    slot_where(|slot| &slot.swapchain, swapchain.as_raw()).is_some_and(|slot| {
        let wanted = slot.wanted_extent.load(Ordering::Acquire);
        wanted != UNKNOWN && wanted != slot.swapchain_extent.load(Ordering::Acquire)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swapchains_go_stale_when_the_wanted_extent_changes() {
        let surface = vk::SurfaceKHR::from_raw(0xfeed_0001);
        let swapchain = vk::SwapchainKHR::from_raw(0xfeed_0002);
        let Some(tracked) = SurfaceExtent::claim(surface) else {
            panic!("a fresh table has free slots");
        };
        assert_eq!(wanted_extent(surface), None);
        record_swapchain(
            surface,
            swapchain,
            vk::Extent2D {
                width: 1,
                height: 1,
            },
        );
        assert!(!is_stale(swapchain));
        tracked.set_wanted(800, 600);
        assert!(is_stale(swapchain));
        record_swapchain(
            surface,
            swapchain,
            vk::Extent2D {
                width: 800,
                height: 600,
            },
        );
        assert!(!is_stale(swapchain));
        assert_eq!(tracked.swapchain_extent(), Some((800, 600)));
        drop(tracked);
        assert_eq!(wanted_extent(surface), None);
        assert!(!is_stale(swapchain));
    }
}
