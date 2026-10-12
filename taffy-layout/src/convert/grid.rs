use layouting::GridTrack;
use taffy::{
    GridPlacement, GridTemplateComponent, Line,
    style_helpers::{FromFr, FromLength, TaffyAuto, TaffyGridSpan},
};

pub(super) fn template_tracks(tracks: &[GridTrack]) -> Vec<GridTemplateComponent<String>> {
    tracks.iter().map(|&track| template_track(track)).collect()
}

fn template_track(track: GridTrack) -> GridTemplateComponent<String> {
    match track {
        GridTrack::Length(length) => GridTemplateComponent::from_length(length),
        GridTrack::Fraction(share) => GridTemplateComponent::from_fr(share),
        GridTrack::Auto => GridTemplateComponent::AUTO,
    }
}

pub(super) fn span_placement(span: u16) -> Line<GridPlacement<String>> {
    Line::from_span(span.max(1))
}
