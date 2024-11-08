use super::{segments::Segments, AnalyzedRoute};
use gpx::Waypoint;

use std::fs::OpenOptions;

pub type SegmentsPtr = Vec<Vec<*const Waypoint>>;
pub fn ptrize(segments: &Segments) -> SegmentsPtr {
    segments
        .iter()
        .map(|s| s.iter().map(|w| -> *const Waypoint { *w }).collect())
        .collect()
}

pub fn save_analyzed_route(piste: &AnalyzedRoute, filename: &str) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)
        .unwrap();
    serde_json::to_writer_pretty(file, &piste).unwrap();
}
