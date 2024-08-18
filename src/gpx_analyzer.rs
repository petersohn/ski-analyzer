use gpx::{Gpx, Time};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::ski_area::{Lift, SkiArea};

use segments::{get_segments, Segments};

mod segments;
#[cfg(test)]
mod segments_test;
mod use_lift;

#[derive(Debug)]
pub enum LiftEnd {
    Unknown,
    EndStation,
    Midstation(usize),
}

#[derive(Debug)]
pub struct UseLift<'s> {
    lift: &'s Lift,
    begin_time: Option<OffsetDateTime>,
    end_time: Option<OffsetDateTime>,
    begin_station: Option<LiftEnd>,
    end_station: Option<LiftEnd>,
    is_reverse: bool,
}

#[derive(Debug)]
pub enum ActivityType<'s> {
    Unknown,
    UseLift(UseLift<'s>),
}

struct Activity<'s, 'g> {
    type_: ActivityType<'s>,
    route: Segments<'g>,
}

fn to_odt(time: Option<Time>) -> Option<OffsetDateTime> {
    time.map(|t| t.into())
}

fn format_time_option(time: Option<OffsetDateTime>) -> String {
    match time {
        Some(t) => format!("{}", t.format(&Iso8601::DEFAULT).unwrap()),
        None => "unknown".to_string(),
    }
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
