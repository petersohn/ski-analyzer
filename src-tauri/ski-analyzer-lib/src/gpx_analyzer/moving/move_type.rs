use geo::{Distance, Haversine};
use gpx::Waypoint;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use super::process::Candidate;
use crate::utils::collection::Avg;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MoveType {
    Ski,
    Wait,
    Climb,
    Traverse,
}

#[derive(Debug, Clone, Copy)]
pub struct MinMax<T>
where
    T: std::fmt::Debug + Clone + Copy,
{
    pub min: T,
    pub max: T,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstraintType {
    Speed(MinMax<f64>),
    Inclination(MinMax<f64>),
}

impl ConstraintType {
    pub fn speed(min: f64, max: f64) -> Self {
        ConstraintType::Speed(MinMax { min, max })
    }
    pub fn inclination(min: f64, max: f64) -> Self {
        ConstraintType::Inclination(MinMax { min, max })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConstraintLimit {
    Distance(f64),
    Time(Duration),
}

#[derive(Debug, Clone, Copy)]
pub struct Constraint {
    pub type_: ConstraintType,
    pub limit: ConstraintLimit,
}

impl Constraint {
    pub fn new(type_: ConstraintType, limit: ConstraintLimit) -> Self {
        Constraint { type_, limit }
    }

    //fn evaluate(&self, wp1: &Waypoint, wp2: &Waypoint) -> Option<bool> {
    //    let should_evaluate = match self.limit {
    //        ConstraintLimit::Distance(d) => {
    //            Haversine::distance(wp1.point(), wp2.point()) <= d
    //        }
    //        ConstraintLimit::Time(t) => match (wp1.time, wp2.time) {
    //            (Some(t1), Some(t2)) => {
    //                let duration =
    //                    OffsetDateTime::from(t2) - OffsetDateTime::from(t1);
    //                duration <= t
    //            }
    //            _ => false,
    //        },
    //    };
    //
    //    if !should_evaluate {
    //        return None;
    //    }
    //
    //    let ret = match self.type_ {
    //        ConstraintType::Speed(s) => {
    //            let speed = get_speed(wp1, wp2)?;
    //            speed >= s.min && speed <= s.max
    //        }
    //        ConstraintType::Inclination(s) => {
    //            let inclination = get_inclination(wp1, wp2)?;
    //            inclination >= s.min && inclination <= s.max
    //        }
    //    };
    //
    //    Some(ret)
    //}
}

#[derive(Debug)]
pub struct SimpleCandidate {
    constraints: Vec<Constraint>,
    previous_point: Option<Waypoint>,
    speed: Avg,
    inclination: Avg,
}

impl SimpleCandidate {
    pub fn new(constraints: impl Into<Vec<Constraint>>) -> Self {
        SimpleCandidate {
            constraints: constraints.into(),
            previous_point: None,
            speed: Avg::new(),
            inclination: Avg::new(),
        }
    }
}

//impl Candidate for SimpleCandidate {
//    fn add_point(&mut self, wp: &Waypoint) -> bool {
//        if let Some(prev) = &self.previous_point {
//            let distance = Haversine::distance(prev.point(), wp.point());
//            match (wp1.time, wp2.time) {
//                (Some(t1), Some(t2)) =>
//            };
//        }
//
//        self.previous_point = Some(wp.clone());
//        true
//    }
//}
