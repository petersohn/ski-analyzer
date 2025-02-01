use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

use geo::{Distance, Haversine};
use gpx::Waypoint;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use super::process::Candidate;
use crate::gpx_analyzer::{get_time_diff, DerivedData};
use crate::utils::collection::Avg;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MoveType {
    Ski,
    Wait,
    Climb,
    Traverse,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstraintType {
    Speed,
    Inclination,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstraintLimit {
    Distance,
    Time,
}

#[derive(Debug, Clone, Copy)]
pub struct Constraint {
    pub type_: ConstraintType,
    pub min: f64,
    pub max: f64,
    pub limit_type: ConstraintLimit,
    pub limit: f64,
}

impl Constraint {
    pub fn new(
        type_: ConstraintType,
        min: f64,
        max: f64,
        limit_type: ConstraintLimit,
        limit: f64,
    ) -> Self {
        Constraint {
            type_,
            min,
            max,
            limit_type,
            limit,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LineData {
    data: DerivedData,
    distance: f64,
    time_diff: f64,
}

impl LineData {
    fn get_extent(&self, limit_type: ConstraintLimit) -> f64 {
        match limit_type {
            ConstraintLimit::Time => self.time_diff,
            ConstraintLimit::Distance => self.distance,
        }
    }
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

    fn add(&mut self, value: f64, distance: f64, line_data: &LineData) {
        self.value.add2(value, distance);
        self.extent += line_data.get_extent(self.constraint.limit_type);
    }

    fn trim(&mut self, line_data: &[LineData]) {
        while self.first_id < line_data.len() {
            let data_to_remove = line_data[self.first_id];
            let extent = data_to_remove.get_extent(self.constraint.limit_type);

            if self.extent - extent < self.constraint.limit {
                break;
            }

            let value_ = get_value(&data_to_remove.data, self.constraint.type_);
            if let Some(value) = value_ {
                self.value.remove2(value, data_to_remove.distance);
                self.extent -= extent;
            }
            self.first_id += 1;
        }
    }
}

pub struct SimpleCandidate {
    constraints: Vec<ConstraintAggregate>,
    line_data: Vec<LineData>,
}

impl SimpleCandidate {
    pub fn new(constraints: impl IntoIterator<Item = Constraint>) -> Self {
        SimpleCandidate {
            constraints: constraints
                .into_iter()
                .map(|c| ConstraintAggregate::new(c))
                .collect(),
            line_data: Vec::new(),
        }
    }
}

fn get_value(data: &DerivedData, type_: ConstraintType) -> Option<f64> {
    match type_ {
        ConstraintType::Speed => data.speed,
        ConstraintType::Inclination => data.inclination,
    }
}

impl Candidate for SimpleCandidate {
    fn add_line(&mut self, wp0: &Waypoint, wp1: &Waypoint) -> Option<bool> {
        let (data, distance) = DerivedData::calculate_inner(wp0, wp1);
        let line_data = LineData {
            data,
            distance,
            time_diff: match get_time_diff(wp0, wp1) {
                Some(dt) => dt.as_seconds_f64(),
                None => 0.0,
            },
        };
        self.line_data.push(line_data);

        for agg in &mut self.constraints {
            let value_ = get_value(&data, agg.constraint.type_);
            if let Some(value) = value_ {
                agg.add(value, distance, &line_data);
                agg.trim(&self.line_data);
            }
        }

        Some(true)
    }
}
