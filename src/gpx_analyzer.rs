use gpx::{Gpx, Time};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::ski_area::{Lift, SkiArea};
use use_lift::find_lift_usage;

use segments::{get_segments, Segments};

mod segments;
#[cfg(test)]
mod segments_test;
mod use_lift;

pub type LiftEnd = Option<usize>;

#[derive(Debug)]
pub struct UseLift<'s> {
    lift: &'s Lift,
    begin_time: Option<OffsetDateTime>,
    end_time: Option<OffsetDateTime>,
    begin_station: LiftEnd,
    end_station: LiftEnd,
    is_reverse: bool,
}

#[derive(Debug, Default)]
pub enum ActivityType<'s> {
    #[default]
    Unknown,
    UseLift(UseLift<'s>),
}

#[derive(Debug, Default)]
pub struct Activity<'s, 'g> {
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

pub fn analyze_route<'s, 'g>(
    ski_area: &'s SkiArea,
    gpx: &'g Gpx,
) -> Vec<Activity<'s, 'g>> {
    let segments = get_segments(&gpx);
    // println!("{:#?}", segments);
    let result = use_lift::find_lift_usage(ski_area, &segments);
    result
}