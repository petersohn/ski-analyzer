use gpx::{Gpx, Time};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::ski_area::SkiArea;
use use_lift::find_lift_usage;

use segments::get_segments;

mod segments;
mod use_lift;

#[cfg(test)]
mod segments_test;
#[cfg(test)]
mod test_util;
#[cfg(test)]
mod use_lift_test;

pub use use_lift::{Activity, ActivityType, LiftEnd, UseLift};

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
    let result = find_lift_usage(ski_area, &segments);
    result
}
