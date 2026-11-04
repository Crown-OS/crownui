use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use ash::vk::{self, Handle};

const SLOT_COUNT: usize = 32;
const UNKNOWN: u64 = 0;
const TRIPLE_BUFFER_IMAGES: u32 = 3;

struct Slot {
    surface: AtomicU64,
    wanted_extent: AtomicU64,
    swapchain: AtomicU64,
    swapchain_extent: AtomicU64,
    driver_min_images: AtomicU32,
    mailbox: AtomicBool,
}

impl Slot {
    const fn empty() -> Self {
        Self {
            surface: AtomicU64::new(UNKNOWN),
            wanted_extent: AtomicU64::new(UNKNOWN),
            swapchain: AtomicU64::new(UNKNOWN),
            swapchain_extent: AtomicU64::new(UNKNOWN),
            driver_min_images: AtomicU32::new(0),
            mailbox: AtomicBool::new(false),
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
        self.slot.driver_min_images.store(0, Ordering::Relaxed);
        self.slot.mailbox.store(false, Ordering::Relaxed);
        self.slot.surface.store(UNKNOWN, Ordering::Release);
    }
}

pub(super) fn wanted_extent(surface: vk::SurfaceKHR) -> Option<vk::Extent2D> {
    slot_where(|slot| &slot.surface, surface.as_raw())
        .and_then(|slot| unpack(slot.wanted_extent.load(Ordering::Acquire)))
}

pub(super) fn record_driver_min_images(surface: vk::SurfaceKHR, min_images: u32) {
    if let Some(slot) = slot_where(|slot| &slot.surface, surface.as_raw()) {
        slot.driver_min_images.store(min_images, Ordering::Release);
    }
}

/// Impeller asks for one image more than the driver needs; triple buffering is enough to keep
/// FIFO presentation pipelined, and every image beyond it is a full surface of memory.
pub(super) fn swapchain_image_count(surface: vk::SurfaceKHR, requested: u32) -> u32 {
    slot_where(|slot| &slot.surface, surface.as_raw())
        .map(|slot| slot.driver_min_images.load(Ordering::Acquire))
        .filter(|&driver_min| driver_min > 0)
        .map_or(requested, |driver_min| {
            requested.min(driver_min.max(TRIPLE_BUFFER_IMAGES))
        })
}

pub(super) fn record_mailbox_support(surface: vk::SurfaceKHR, supported: bool) {
    if let Some(slot) = slot_where(|slot| &slot.surface, surface.as_raw()) {
        slot.mailbox.store(supported, Ordering::Release);
    }
}

/// The render thread paces itself by frame callbacks, so a swapchain presents in mailbox mode
/// where the surface allows it: FIFO would add a compositor-side barrier that can hold an idle
/// window's next commit, and the frame callback it carries, until something else repaints.
pub(super) fn swapchain_present_mode(
    surface: vk::SurfaceKHR,
    requested: vk::PresentModeKHR,
) -> vk::PresentModeKHR {
    slot_where(|slot| &slot.surface, surface.as_raw())
        .filter(|slot| slot.mailbox.load(Ordering::Acquire))
        .map_or(requested, |_| vk::PresentModeKHR::MAILBOX)
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

    #[test]
    fn swapchains_keep_the_driver_minimum_but_at_least_three_images() {
        let surface = vk::SurfaceKHR::from_raw(0xfeed_0003);
        let Some(tracked) = SurfaceExtent::claim(surface) else {
            panic!("a fresh table has free slots");
        };
        assert_eq!(swapchain_image_count(surface, 4), 4);
        record_driver_min_images(surface, 3);
        assert_eq!(swapchain_image_count(surface, 4), 3);
        record_driver_min_images(surface, 2);
        assert_eq!(swapchain_image_count(surface, 4), 3);
        assert_eq!(swapchain_image_count(surface, 2), 2);
        record_driver_min_images(surface, 5);
        assert_eq!(swapchain_image_count(surface, 6), 5);
        drop(tracked);
        assert_eq!(swapchain_image_count(surface, 4), 4);
    }

    #[test]
    fn swapchains_present_in_mailbox_mode_only_where_the_surface_supports_it() {
        let surface = vk::SurfaceKHR::from_raw(0xfeed_0004);
        let fifo = vk::PresentModeKHR::FIFO;
        let Some(tracked) = SurfaceExtent::claim(surface) else {
            panic!("a fresh table has free slots");
        };
        assert_eq!(swapchain_present_mode(surface, fifo), fifo);
        record_mailbox_support(surface, true);
        assert_eq!(
            swapchain_present_mode(surface, fifo),
            vk::PresentModeKHR::MAILBOX
        );
        record_mailbox_support(surface, false);
        assert_eq!(swapchain_present_mode(surface, fifo), fifo);
        record_mailbox_support(surface, true);
        drop(tracked);
        assert_eq!(swapchain_present_mode(surface, fifo), fifo);
    }
}
