use super::segments::Segments;
use gpx::Waypoint;

pub type SegmentsPtr = Vec<Vec<*const Waypoint>>;
pub fn ptrize(segments: &Segments) -> SegmentsPtr {
    segments
        .iter()
        .map(|s| s.iter().map(|w| -> *const Waypoint { *w }).collect())
        .collect()
}
