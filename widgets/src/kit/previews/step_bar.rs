use view::Styled;

use crate::kit::{
    preview::{Preview, stage},
    step_bar,
};

pub(super) fn previews() -> Vec<Preview> {
    vec![
        Preview {
            name: "step_bar_first_of_four",
            build: || stage(step_bar(4, 0_usize).w(360.0)),
        },
        Preview {
            name: "step_bar_third_of_four",
            build: || stage(step_bar(4, 2_usize).w(360.0)),
        },
    ]
}
