use crowntest::TestApp;
use widgets::kit::previews;

#[test]
fn every_preview_builds_and_settles_into_a_stable_snapshot() {
    for preview in previews() {
        let mut app = TestApp::with_size(preview.view(), 480.0, 320.0);
        app.settle();
        insta::with_settings!({ snapshot_suffix => preview.name }, {
            insta::assert_snapshot!("preview", app.snapshot());
        });
    }
}

#[test]
fn preview_names_are_unique() {
    let mut names: Vec<_> = previews().iter().map(|preview| preview.name).collect();
    names.sort_unstable();
    let total = names.len();
    names.dedup();
    assert_eq!(names.len(), total);
}
