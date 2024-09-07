use super::segments::Segments;
use super::use_lift::find_lift_usage;
use super::{Activity, ActivityType};
use crate::ski_area::{BoundedGeometry, Lift, PointWithElevation, SkiArea};
use crate::test_util::{init, Init};

use geo::{coord, point, LineString};
use gpx::{Gpx, Track, TrackSegment, Waypoint};
use rstest::{fixture, rstest};
use time::OffsetDateTime;

fn line(input: &[(f64, f64)]) -> LineString {
    LineString::new(
        input.iter().map(|(x, y)| coord! { x: *x, y: *y }).collect(),
    )
}

fn lift(
    name: String,
    line: LineString,
    midstations: &[usize],
    can_go_reverse: bool,
    can_disembark: bool,
) -> Lift {
    let stations = [0]
        .iter()
        .chain(midstations.iter())
        .chain([line.0.len() - 1].iter())
        .map(|i| PointWithElevation {
            point: line[*i].into(),
            elevation: 0,
        })
        .collect();
    Lift {
        ref_: String::new(),
        name,
        type_: String::new(),
        line: BoundedGeometry::new(line).unwrap(),
        stations,
        can_go_reverse,
        can_disembark,
    }
}

fn ski_area(lifts: Vec<Lift>) -> SkiArea {
    SkiArea {
        name: String::new(),
        lifts,
        pistes: Vec::new(),
    }
}

fn segment(input: &[(f64, f64, Option<OffsetDateTime>)]) -> TrackSegment {
    let mut result = TrackSegment::new();
    result.points = input
        .iter()
        .map(|(x, y, t)| {
            let mut result = Waypoint::new(point! { x: *x, y: *y });
            result.time = t.map(|tt| tt.into());
            result
        })
        .collect();
    result
}

fn make_gpx(input: Vec<TrackSegment>) -> Gpx {
    let mut track = Track::new();
    track.segments = input;
    let mut result = Gpx::default();
    result.tracks = vec![track];
    result
}

fn get_segments<'g>(gpx: &'g Gpx) -> Segments<'g> {
    gpx.tracks
        .iter()
        .map(|t| t.segments.iter().map(|s| s.points.iter().collect()))
        .flatten()
        .collect()
}

#[fixture]
fn line00() -> LineString {
    line(&[
        (6.6532147, 45.3865145),
        (6.6531819, 45.3855226),
        (6.6531714, 45.3852169),
        (6.6529913, 45.3799574),
        (6.6528696, 45.3729557),
    ])
}

#[fixture]
fn line01() -> LineString {
    line(&[
        (6.6531285, 45.3865073),
        (6.6530753, 45.3855296),
        (6.6530649, 45.3852277),
        (6.6528845, 45.3799605),
        (6.652721, 45.3729675),
    ])
}

#[rstest]
fn simple(_init: Init, line00: LineString) {
    let s =
        ski_area(vec![lift("Lift 1".to_string(), line00, &[], false, false)]);
    let g = make_gpx(vec![segment(&[
        (6.6534126, 45.3866878, None),
        (6.6532833, 45.386625, None),
        (6.6532399, 45.3865363, None),
        (6.653222, 45.3864292, None),
        (6.6532228, 45.3862921, None),
        (6.6532124, 45.3861493, None),
        (6.6531947, 45.3859249, None),
        (6.6532057, 45.3857614, None),
        (6.6532034, 45.3855642, None),
        (6.6532004, 45.3853605, None),
        (6.6531879, 45.3851181, None),
        (6.6531941, 45.384915, None),
        (6.6532009, 45.384639, None),
        (6.6531769, 45.3843765, None),
        (6.653174, 45.3841198, None),
        (6.6531553, 45.3838894, None),
        (6.6531506, 45.3836928, None),
        (6.6531405, 45.3834303, None),
        (6.6531397, 45.3831716, None),
        (6.653134, 45.3829065, None),
        (6.6531375, 45.3826413, None),
        (6.6530994, 45.3824098, None),
        (6.6531095, 45.382125, None),
        (6.65308, 45.3818723, None),
        (6.6530781, 45.3815575, None),
        (6.6530689, 45.3812823, None),
        (6.6530651, 45.3810034, None),
        (6.6530589, 45.3806956, None),
        (6.6530332, 45.3803867, None),
        (6.6530387, 45.3800697, None),
        (6.6530319, 45.3797959, None),
        (6.6530351, 45.3795241, None),
        (6.6530443, 45.3791876, None),
        (6.6530295, 45.3788297, None),
        (6.6529942, 45.3785015, None),
        (6.6529933, 45.3782173, None),
        (6.6529849, 45.3779231, None),
        (6.6529663, 45.3776285, None),
        (6.652974, 45.3773365, None),
        (6.6529769, 45.3770948, None),
        (6.6529653, 45.3767615, None),
        (6.6529576, 45.3763565, None),
        (6.6529501, 45.3759228, None),
        (6.6529244, 45.37554, None),
        (6.6529276, 45.375154, None),
        (6.6529177, 45.3748065, None),
        (6.6529115, 45.3743625, None),
        (6.6528976, 45.3738913, None),
        (6.6528989, 45.3735875, None),
        (6.6529144, 45.3732073, None),
        (6.6528952, 45.3730052, None),
        (6.6528336, 45.3728979, None),
        (6.652673, 45.3728196, None),
        (6.6524959, 45.3727732, None),
    ])]);

    let actual = find_lift_usage(&s, &get_segments(&g));
}
