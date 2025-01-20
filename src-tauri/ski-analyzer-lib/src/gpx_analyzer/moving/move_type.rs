use std::collections::HashMap;
use std::rc::Rc;

use geo::{Distance, Haversine};
use gpx::Waypoint;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use super::process::Candidate;
use crate::gpx_analyzer::DerivedData;
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
}

#[derive(Debug, Clone, Copy)]
struct LineData {
    data: DerivedData,
    distance: f64,
}

struct ConstraintAggregate {
    constraint: Constraint,
    first_id: usize,
    value: Avg,
    extent: f64,
}

impl ConstraintAggregate {
    fn new(constraint: Constraint) -> Self {
        Self {
            constraint,
            first_id: 0,
            value: Avg::new(),
            extent: 0.0,
        }
    }
}

pub struct SimpleCandidate {
    constraints: Vec<ConstraintAggregate>,
    previous_point: Option<Waypoint>,
    line_data: Vec<LineData>,
}

impl SimpleCandidate {
    pub fn new(constraints: impl IntoIterator<Item = Constraint>) -> Self {
        SimpleCandidate {
            constraints: constraints
                .into_iter()
                .map(|c| ConstraintAggregate::new(c))
                .collect(),
            previous_point: None,
            line_data: Vec::new(),
        }
    }

    fn check_constraint(&mut self, agg: &mut ConstraintAggregate) -> bool {
        let value = match &agg.constraint.type_ {
            ConstraintType::Speed(_) => line_data.data.speed,
            ConstraintType::Inclination(_) => line_data.data.inclination,
        };

        //let (extent, limit) = match &agg.constraint.limit {
        //    //ConstraintLimit::Time(_) => (
        //}

        if let Some(v) = value {
            agg.value.add2(v, line_data.distance);
        }
    }
}

impl Candidate for SimpleCandidate {
    fn add_point(&mut self, wp: &Waypoint) -> bool {
        if let Some(prev) = &self.previous_point {
            let (data, distance) = DerivedData::calculate_inner(prev, wp);
            let line_data = LineData { data, distance };
            self.line_data.push(line_data);

            for agg in &mut self.constraints {
                if !self.check_constraint(agg) {
                    return false;
                }
            }
        }

        self.previous_point = Some(wp.clone());
        true
    }
}
