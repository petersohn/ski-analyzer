use gpx::Gpx;
use time::OffsetDateTime;

use super::ski_area::{Lift, SkiArea};

pub struct UseLift<'s> {
    lift: &'s Lift,
    begin_time: OffsetDateTime,
    end_time: OffsetDateTime,
    begin_station: Option<usize>,
    end_station: Option<usize>,
}

pub enum ActivityType<'s> {
    UseLift(UseLift<'s>),
}

pub fn analyze_route<'s>(
    ski_area: &'s SkiArea,
    gpx: &Gpx,
) -> Vec<ActivityType<'s>> {
    let result = Vec::new();
    result
}
