use super::waypoint_ser::WaypointDef;
use super::{format_time_option, to_odt};
use crate::config::get_config;
use crate::error::{Error, ErrorType, Result};
use crate::utils::bounded_geometry::BoundedGeometry;
use crate::utils::rect::union_rects_if;

use geo::{Coord, Rect};
use gpx::{Gpx, Time, Waypoint};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::mem;

pub type Segment = Vec<Waypoint>;

pub type SegmentCoordinate = (usize, usize);

#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Segments(pub Vec<Segment>);

impl Segments {
    pub fn new(segments: Vec<Segment>) -> Self {
        Self { 0: segments }
    }

    pub fn split_end(&mut self, coord: SegmentCoordinate) -> Self {
        let result = if coord.1 == 0 {
            self.0.drain(coord.0..).collect()
        } else {
            let first_segment: Segment =
                self.0[coord.0].drain(coord.1..).collect();
            if coord.0 == self.0.len() - 1 {
                vec![first_segment]
            } else {
                [first_segment]
                    .into_iter()
                    .chain(self.0.drain(coord.0 + 1..))
                    .collect()
            }
        };

        Self::new(result)
    }

    pub fn clone_part(
        &self,
        begin: SegmentCoordinate,
        end: SegmentCoordinate,
    ) -> Self {
        if begin.0 == end.0 {
            return Self::new(vec![self.0[begin.0]
                .get(begin.1..end.1)
                .unwrap()
                .into()]);
        }
        let mut result = Vec::new();
        result.reserve(end.0 - begin.0 + 1);
        result.push(self.0[begin.0].get(begin.1..).unwrap().into());
        for i in (begin.0 + 1)..end.0 {
            result.push(self.0[i].clone());
        }
        result.push(self.0[end.0].get(0..end.1).unwrap().into());
        Self::new(result)
    }
}

impl Serialize for Segments {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data: Vec<Vec<WaypointDef>> = self
            .0
            .iter()
            .map(|s| s.into_iter().map(|wp| wp.clone().into()).collect())
            .collect();
        data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Segments {
    fn deserialize<D>(
        deserializer: D,
    ) -> std::result::Result<Segments, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data: Vec<Vec<WaypointDef>> = Vec::deserialize(deserializer)?;
        Ok(Segments::new(
            data.into_iter()
                .map(|s| s.into_iter().map(|wp| wp.into()).collect())
                .collect(),
        ))
    }
}

const PRECISION_LIMIT: f64 = 10.0;

impl Segments {
    pub fn from_gpx(gpx: Gpx) -> Result<BoundedGeometry<Segments>> {
        let mut result = Vec::new();
        let mut bounding_rect: Option<Rect> = None;
        let config = get_config();

        struct BadPrecisionDebug {
            begin: Option<Time>,
            end: Option<Time>,
            min_precision: f64,
            max_precision: f64,
        }

        for track in gpx.tracks {
            for segment in track.segments {
                let mut add = |current: &mut Vec<Waypoint>| {
                    if !current.is_empty() {
                        result.push(mem::take(current));
                    }
                };
                let mut current = Vec::new();

                let mut bad_precision_debug: Option<BadPrecisionDebug> = None;

                for waypoint in segment.points {
                    let precision = match waypoint.hdop {
                        Some(p) => p,
                        None => 0.0,
                    };
                    if precision > PRECISION_LIMIT {
                        add(&mut current);
                        if config.is_vv() {
                            if let Some(bpd) = bad_precision_debug.as_mut() {
                                bpd.min_precision =
                                    bpd.min_precision.min(precision);
                                bpd.max_precision =
                                    bpd.max_precision.max(precision);
                                bpd.end = waypoint.time;
                            } else {
                                bad_precision_debug = Some(BadPrecisionDebug {
                                    begin: waypoint.time,
                                    end: waypoint.time,
                                    min_precision: precision,
                                    max_precision: precision,
                                });
                            }
                        }
                    } else {
                        if config.is_vv() {
                            if let Some(bpd) = bad_precision_debug.as_ref() {
                                eprintln!(
                                "Bad precision between {} and {}: {} - {} m",
                                format_time_option(to_odt(bpd.begin)),
                                format_time_option(to_odt(bpd.end)),
                                bpd.min_precision,
                                bpd.max_precision
                            );
                            }
                            bad_precision_debug = None;
                        }
                        let coord = Coord::from(waypoint.point());
                        let r0 = Rect::new(coord, coord);
                        bounding_rect = union_rects_if(bounding_rect, Some(r0));
                        current.push(waypoint);
                    }
                }
                add(&mut current);
            }
        }

        Ok(BoundedGeometry {
            item: Segments::new(result),
            bounding_rect: bounding_rect
                .ok_or(Error::new_s(ErrorType::InputError, "Empty route"))?,
        })
    }
}
