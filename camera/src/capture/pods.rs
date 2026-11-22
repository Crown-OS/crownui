use std::io::Cursor;

use pipewire::spa::{
    buffer::DataType,
    param::{
        ParamType,
        format::{FormatProperties, MediaSubtype, MediaType},
        format_utils::parse_format,
        video::{VideoFormat, VideoInfoRaw},
    },
    pod::{self, ChoiceValue, Pod, Property, Value, serialize::PodSerializer},
    sys::{SPA_PARAM_BUFFERS_dataType, spa_format_video_mjpg_parse, spa_video_info_mjpg},
    utils::{Choice, ChoiceEnum, ChoiceFlags, Fraction, Rectangle, SpaTypes},
};

use crate::{CameraError, CameraOptions, CaptureFormat, Encoding};

const MIN_SIZE: Rectangle = Rectangle {
    width: 1,
    height: 1,
};
const MAX_SIZE: Rectangle = Rectangle {
    width: 8192,
    height: 8192,
};
const MAX_FRAME_RATE: u32 = 240;

fn serialize(object: pod::Object) -> Result<Vec<u8>, CameraError> {
    PodSerializer::serialize(Cursor::new(Vec::new()), &Value::Object(object))
        .map(|(cursor, _)| cursor.into_inner())
        .map_err(|_| CameraError::Format)
}

/// Buffers the CPU can read. A camera left to choose may hand out dma-bufs, which a stream that
/// never said it takes them fails to link (`-EINVAL` on `use_buffers`).
pub(crate) fn cpu_buffers() -> Result<Vec<u8>, CameraError> {
    let readable = (1 << DataType::MemPtr.as_raw()) | (1 << DataType::MemFd.as_raw());
    serialize(pod::object!(
        SpaTypes::ObjectParamBuffers,
        ParamType::Buffers,
        Property::new(
            SPA_PARAM_BUFFERS_dataType,
            Value::Choice(ChoiceValue::Int(Choice(
                ChoiceFlags::empty(),
                ChoiceEnum::Flags {
                    default: readable,
                    flags: vec![readable],
                },
            ))),
        ),
    ))
}

/// One `EnumFormat` per encoding, MJPEG first: UVC cameras only reach full frame rate at HD
/// sizes compressed.
pub(crate) fn enum_formats(options: &CameraOptions) -> Result<Vec<Vec<u8>>, CameraError> {
    [
        format_object(options, MediaSubtype::Mjpg, None),
        format_object(options, MediaSubtype::Raw, Some(VideoFormat::YUY2)),
        format_object(options, MediaSubtype::Raw, Some(VideoFormat::NV12)),
    ]
    .into_iter()
    .map(serialize)
    .collect()
}

fn format_object(
    options: &CameraOptions,
    subtype: MediaSubtype,
    raw: Option<VideoFormat>,
) -> pod::Object {
    let size = Rectangle {
        width: options.width,
        height: options.height,
    };
    let rate = Fraction {
        num: options.frame_rate,
        denom: 1,
    };
    let mut object = pod::object!(
        SpaTypes::ObjectParamFormat,
        ParamType::EnumFormat,
        pod::property!(FormatProperties::MediaType, Id, MediaType::Video),
        pod::property!(FormatProperties::MediaSubtype, Id, subtype),
        pod::property!(
            FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            size,
            MIN_SIZE,
            MAX_SIZE
        ),
        pod::property!(
            FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            rate,
            Fraction { num: 0, denom: 1 },
            Fraction {
                num: MAX_FRAME_RATE,
                denom: 1
            }
        ),
    );
    if let Some(format) = raw {
        object
            .properties
            .push(pod::property!(FormatProperties::VideoFormat, Id, format));
    }
    object
}

/// The format a `Format` param fixed, or `None` for one this crate cannot decode.
pub(crate) fn negotiated(param: &Pod) -> Option<CaptureFormat> {
    let (media_type, subtype) = parse_format(param).ok()?;
    if media_type != MediaType::Video {
        return None;
    }
    if subtype == MediaSubtype::Mjpg {
        // SAFETY: zeroed is a valid `spa_video_info_mjpg`, which the parser fills from `param`.
        let mut info: spa_video_info_mjpg = unsafe { std::mem::zeroed() };
        // SAFETY: `param` is a valid pod for the duration of the call.
        let result = unsafe { spa_format_video_mjpg_parse(param.as_raw_ptr(), &mut info) };
        return (result >= 0).then_some(CaptureFormat {
            encoding: Encoding::Mjpeg,
            width: info.size.width,
            height: info.size.height,
        });
    }
    let mut info = VideoInfoRaw::new();
    info.parse(param).ok()?;
    let encoding = match info.format() {
        VideoFormat::YUY2 => Encoding::Yuyv,
        VideoFormat::NV12 => Encoding::Nv12,
        _ => return None,
    };
    Some(CaptureFormat {
        encoding,
        width: info.size().width,
        height: info.size().height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offers_every_decodable_encoding_mjpeg_first() {
        let offers = enum_formats(&CameraOptions::default()).expect("pods serialize");
        assert_eq!(offers.len(), 3);
        let mjpeg = Pod::from_bytes(&offers[0]).expect("a pod");
        let (_, subtype) = parse_format(mjpeg).expect("a format pod");
        assert_eq!(subtype, MediaSubtype::Mjpg);
    }

    #[test]
    fn the_buffers_param_serializes() {
        assert!(cpu_buffers().is_ok_and(|pod| !pod.is_empty()));
    }
}
