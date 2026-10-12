use crownconfig::Keybind;
use view::{AnyView, Element, Semantic, View, component};

use crate::kit::{
    KeybindInputKind, keybind_input,
    preview::{Preview, stage},
};

fn keybind_stage(
    bind: Keybind,
    configure: fn(Element<KeybindInputKind>) -> Element<KeybindInputKind>,
) -> AnyView {
    component(move |cx| {
        let value = cx.signal(bind);
        stage(configure(keybind_input(value)))
    })
    .boxed()
}

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "keybind_input_chord",
            build: || keybind_stage(Keybind::SUPER_SPACE, |field| field),
        },
        Preview {
            name: "keybind_input_unbound",
            build: || keybind_stage(Keybind::NONE, |field| field),
        },
        Preview {
            name: "keybind_input_modifier_only",
            build: || keybind_stage(Keybind::SUPER_CTRL, |field| field),
        },
        Preview {
            name: "keybind_input_disabled",
            build: || keybind_stage(Keybind::SUPER_SPACE, |field| field.disabled(true)),
        },
    ]
}
