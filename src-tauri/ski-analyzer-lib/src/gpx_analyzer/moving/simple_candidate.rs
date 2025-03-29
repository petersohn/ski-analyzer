//use geo::{Distance, Haversine};
//use gpx::Waypoint;
//
//use super::process::{Candidate, CandidateFactory};
//use crate::gpx_analyzer::{get_elevation_diff, get_time_diff};
//
//#[derive(Debug, Clone, Copy)]
//pub enum ConstraintType {
//    Speed,
//    Inclination,
//}
//
//#[derive(Debug, Clone, Copy)]
//pub enum ConstraintLimit {
//    Distance,
//    Time,
//}
//
//#[derive(Debug, Clone, Copy)]
//pub struct Constraint {
//    pub type_: ConstraintType,
//    pub min: Option<f64>,
//    pub max: Option<f64>,
//    pub limit_type: ConstraintLimit,
//    pub limit: f64,
//}
//
//impl Constraint {
//    pub fn new(
//        type_: ConstraintType,
//        min: Option<f64>,
//        max: Option<f64>,
//        limit_type: ConstraintLimit,
//        limit: f64,
//    ) -> Self {
//        Constraint {
//            type_,
//            min,
//            max,
//            limit_type,
//            limit,
//        }
//    }
//}
//
//#[derive(Debug, Clone, Copy)]
//struct LineData {
//    distance: f64,
//    elevation_diff: Option<f64>,
//    time_diff: Option<f64>,
//}
//
//impl LineData {
//    fn get_value(&self, type_: ConstraintType) -> (f64, f64) {
//        match (type_, self.elevation_diff, self.time_diff) {
//            (ConstraintType::Speed, _, Some(dt)) => (self.distance, dt),
//            (ConstraintType::Inclination, Some(de), _) => (de, self.distance),
//            _ => (0.0, 0.0),
//        }
//    }
//
//    fn get_extent(&self, limit_type: ConstraintLimit) -> f64 {
//        match limit_type {
//            ConstraintLimit::Time => self.time_diff.unwrap_or(0.0),
//            ConstraintLimit::Distance => self.distance,
//        }
//    }
//}
//
//struct ConstraintAggregate {
//    constraint: Constraint,
//    first_id: usize,
//    value: (f64, f64),
//    extent: f64,
//}
//
//impl ConstraintAggregate {
//    fn new(constraint: Constraint) -> Self {
//        Self {
//            constraint,
//            first_id: 0,
//            value: (0.0, 0.0),
//            extent: 0.0,
//        }
//    }
//
//    fn add(&mut self, line_data: &LineData) {
//        let value = line_data.get_value(self.constraint.type_);
//        self.value.0 += value.0;
//        self.value.1 += value.1;
//        self.extent += line_data.get_extent(self.constraint.limit_type);
//    }
//
//    fn trim(&mut self, line_data: &[LineData]) {
//        while self.first_id < line_data.len() {
//            let data_to_remove = line_data[self.first_id];
//            let extent = data_to_remove.get_extent(self.constraint.limit_type);
//
//            if self.extent - extent < self.constraint.limit {
//                break;
//            }
//
//            let value = data_to_remove.get_value(self.constraint.type_);
//            self.value.0 -= value.0;
//            self.value.1 -= value.1;
//            self.extent -= extent;
//            self.first_id += 1;
//        }
//    }
//
//    fn evaluate(&self) -> Option<bool> {
//        if self.extent < self.constraint.limit || self.value.1 == 0.0 {
//            return None;
//        }
//
//        let value = self.value.0 / self.value.1;
//        Some(
//            self.constraint.min.map_or(true, |m| value >= m)
//                && self.constraint.max.map_or(true, |m| value <= m),
//        )
//    }
//}
//
//pub struct SimpleCandidate {
//    constraints: Vec<ConstraintAggregate>,
//    line_data: Vec<LineData>,
//}
//
//impl SimpleCandidate {
//    pub fn new(constraints: impl IntoIterator<Item = Constraint>) -> Self {
//        SimpleCandidate {
//            constraints: constraints
//                .into_iter()
//                .map(|c| ConstraintAggregate::new(c))
//                .collect(),
//            line_data: Vec::new(),
//        }
//    }
//}
//
//impl Candidate for SimpleCandidate {
//    fn add_line(&mut self, wp0: &Waypoint, wp1: &Waypoint) -> Option<bool> {
//        let distance = Haversine::distance(wp0.point(), wp1.point());
//        let line_data = LineData {
//            distance,
//            elevation_diff: get_elevation_diff(wp0, wp1),
//            time_diff: get_time_diff(wp0, wp1).map(|d| d.as_seconds_f64()),
//        };
//        self.line_data.push(line_data);
//
//        let mut has_not_evaluatable_constraint = false;
//
//        for agg in &mut self.constraints {
//            agg.add(&line_data);
//            agg.trim(&self.line_data);
//
//            match agg.evaluate() {
//                None => has_not_evaluatable_constraint = true,
//                Some(false) => return Some(false),
//                Some(true) => (),
//            };
//        }
//
//        if has_not_evaluatable_constraint {
//            None
//        } else {
//            Some(true)
//        }
//    }
//}
//
//pub struct SimpleCandidateFactory {
//    constraints: Vec<Constraint>,
//}
//
//impl SimpleCandidateFactory {
//    pub fn new(constraints: Vec<Constraint>) -> Box<dyn CandidateFactory> {
//        Box::new(Self { constraints })
//    }
//}
//
//impl CandidateFactory for SimpleCandidateFactory {
//    fn create_candidate(&self) -> Box<dyn Candidate> {
//        Box::new(SimpleCandidate::new(self.constraints.clone()))
//    }
//}
