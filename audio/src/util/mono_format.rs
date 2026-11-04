use std::io::Cursor;

use pipewire::spa::{
    param::{
        ParamType,
        audio::{AudioFormat, AudioInfoRaw, MAX_CHANNELS},
    },
    pod::{Object, Value, serialize::PodSerializer},
    sys::SPA_AUDIO_CHANNEL_MONO,
    utils::SpaTypes,
};

use crate::{AudioError, SAMPLE_RATE};

/// The `EnumFormat` pod for mono `f32` at 48 kHz, which PipeWire converts any source to.
pub(crate) fn mono_format() -> Result<Vec<u8>, AudioError> {
    let mut info = AudioInfoRaw::new();
    info.set_format(AudioFormat::F32LE);
    info.set_rate(SAMPLE_RATE);
    info.set_channels(1);
    let mut position = [0; MAX_CHANNELS];
    position[0] = SPA_AUDIO_CHANNEL_MONO;
    info.set_position(position);
    let format = Value::Object(Object {
        type_: SpaTypes::ObjectParamFormat.as_raw(),
        id: ParamType::EnumFormat.as_raw(),
        properties: info.into(),
    });
    PodSerializer::serialize(Cursor::new(Vec::new()), &format)
        .map(|(cursor, _)| cursor.into_inner())
        .map_err(|_| AudioError::Format)
}

#[cfg(test)]
mod tests {
    use pipewire::spa::pod::Pod;

    use super::*;

    #[test]
    fn the_format_is_a_valid_pod() {
        let bytes = mono_format().expect("serialise the format");
        assert!(Pod::from_bytes(&bytes).is_some());
    }
}
