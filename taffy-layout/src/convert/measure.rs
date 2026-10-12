use layouting::{AvailableSpace, MeasureInput};

pub(crate) fn measure_input(
    known_dimensions: taffy::Size<Option<f32>>,
    available_space: taffy::Size<taffy::AvailableSpace>,
) -> MeasureInput {
    MeasureInput {
        known_width: known_dimensions.width,
        known_height: known_dimensions.height,
        available_width: available(available_space.width),
        available_height: available(available_space.height),
    }
}

fn available(space: taffy::AvailableSpace) -> AvailableSpace {
    match space {
        taffy::AvailableSpace::Definite(length) => AvailableSpace::Definite(length),
        taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
        taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
    }
}

pub(crate) fn root_available_space(size: geometry::Size) -> taffy::Size<taffy::AvailableSpace> {
    let axis = |length: f32| {
        if length.is_finite() {
            taffy::AvailableSpace::Definite(length)
        } else {
            taffy::AvailableSpace::MaxContent
        }
    };
    taffy::Size {
        width: axis(size.width),
        height: axis(size.height),
    }
}

pub(crate) fn taffy_size(size: geometry::Size) -> taffy::Size<f32> {
    taffy::Size {
        width: size.width,
        height: size.height,
    }
}
