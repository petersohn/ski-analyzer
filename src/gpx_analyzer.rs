use gpx::{Gpx, Time};
use time::OffsetDateTime;

use crate::ski_area::{Lift, SkiArea};

use segments::{get_segments, Segments};

mod segments;

#[derive(Debug)]
pub enum LiftEnd {
    Unknown,
    EndStation,
    Midstation(usize),
}

#[derive(Debug)]
pub struct UseLift<'s> {
    lift: &'s Lift,
    begin_time: OffsetDateTime,
    end_time: OffsetDateTime,
    begin_station: Option<LiftEnd>,
    end_station: Option<LiftEnd>,
    is_reverse: bool,
}

#[derive(Debug)]
pub enum ActivityType<'s> {
    UseLift(UseLift<'s>),
}

struct Activity<'s, 'g> {
    type_: Option<ActivityType<'s>>,
    route: Segments<'g>,
}

pub fn analyze_route<'s>(
    ski_area: &'s SkiArea,
    gpx: &Gpx,
) -> Vec<ActivityType<'s>> {
    let mut result = Vec::new();

    let segments = get_segments(&gpx);
    println!("{:#?}", segments);

    result
}
