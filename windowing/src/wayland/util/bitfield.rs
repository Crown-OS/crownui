use bitflags::Flags;
use sctk::reexports::client::WEnum;

/// Keeps the flags this protocol version knows, dropping bits a newer compositor added.
pub(crate) fn known_flags<T: Flags<Bits = u32>>(value: WEnum<T>) -> T {
    match value {
        WEnum::Value(flags) => flags,
        WEnum::Unknown(bits) => T::from_bits_truncate(bits),
    }
}
