use std::{mem, ptr};

const PERF_TYPE_HARDWARE: u32 = 0;
const PERF_COUNT_HW_INSTRUCTIONS: u64 = 1;
const PERF_FLAG_FD_CLOEXEC: libc::c_ulong = 1 << 3;
const PERF_EVENT_IOC_ENABLE: libc::c_ulong = 0x2400;
const PERF_EVENT_IOC_DISABLE: libc::c_ulong = 0x2401;
const PERF_EVENT_IOC_RESET: libc::c_ulong = 0x2403;
/// `disabled`, `exclude_kernel` and `exclude_hv` in `perf_event_attr`'s flag word.
const ATTR_FLAGS: u64 = 1 | (1 << 5) | (1 << 6);

/// `struct perf_event_attr` up to `config3` (`PERF_ATTR_SIZE_VER8`); the kernel accepts any
/// size it knows.
#[repr(C)]
struct PerfEventAttr {
    kind: u32,
    size: u32,
    config: u64,
    sample_period: u64,
    sample_type: u64,
    read_format: u64,
    flags: u64,
    wakeup_events: u32,
    bp_type: u32,
    bp_addr: u64,
    bp_len: u64,
    branch_sample_type: u64,
    sample_regs_user: u64,
    sample_stack_user: u32,
    clockid: i32,
    sample_regs_intr: u64,
    aux_watermark: u32,
    sample_max_stack: u16,
    reserved: u16,
    aux_sample_size: u32,
    reserved2: u32,
    sig_data: u64,
    config3: u64,
}

/// Counts the calling thread's retired user-space instructions, when the kernel lets it
/// (`perf_event_paranoid` at most 2). Instruction counts barely vary between runs, unlike time.
pub(crate) struct Instructions {
    descriptor: Option<libc::c_int>,
}

impl Instructions {
    pub(crate) fn open() -> Self {
        let mut attr: PerfEventAttr = unsafe { mem::zeroed() };
        attr.kind = PERF_TYPE_HARDWARE;
        attr.size = mem::size_of::<PerfEventAttr>() as u32;
        attr.config = PERF_COUNT_HW_INSTRUCTIONS;
        attr.flags = ATTR_FLAGS;
        let descriptor = unsafe {
            libc::syscall(
                libc::SYS_perf_event_open,
                ptr::from_ref(&attr),
                0 as libc::pid_t,
                -1 as libc::c_int,
                -1 as libc::c_int,
                PERF_FLAG_FD_CLOEXEC,
            )
        };
        Self {
            descriptor: libc::c_int::try_from(descriptor)
                .ok()
                .filter(|descriptor| *descriptor >= 0),
        }
    }

    pub(crate) fn start(&mut self) {
        if let Some(descriptor) = self.descriptor {
            unsafe {
                libc::ioctl(descriptor, PERF_EVENT_IOC_RESET, 0);
                libc::ioctl(descriptor, PERF_EVENT_IOC_ENABLE, 0);
            }
        }
    }

    pub(crate) fn stop(&mut self) -> Option<u64> {
        let descriptor = self.descriptor?;
        let mut count: u64 = 0;
        let read = unsafe {
            libc::ioctl(descriptor, PERF_EVENT_IOC_DISABLE, 0);
            libc::read(
                descriptor,
                ptr::from_mut(&mut count).cast(),
                mem::size_of::<u64>(),
            )
        };
        (read == mem::size_of::<u64>() as isize).then_some(count)
    }
}

impl Drop for Instructions {
    fn drop(&mut self) {
        if let Some(descriptor) = self.descriptor.take() {
            unsafe {
                libc::close(descriptor);
            }
        }
    }
}
