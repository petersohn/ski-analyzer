use super::waypoint_ser::WaypointDef;
use super::{Activity, ActivityType};
use crate::error::Result;

use gpx::Waypoint;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::mem::take;

pub type Segment = Vec<Waypoint>;

pub type SegmentCoordinate = (usize, usize);

#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Segments(pub Vec<Segment>);

impl Segments {
    pub fn new(segments: Vec<Segment>) -> Self {
        Self { 0: segments }
    }

    pub fn split_end(&mut self, mut coord: SegmentCoordinate) -> Self {
        coord = self.get_closest_valid_coord(coord);
        let result = if coord.1 == 0 {
            self.0.drain(coord.0..).collect()
        } else {
            let segment_in = &mut self.0[coord.0];
            let segment_out: Segment = [segment_in[coord.1].clone()]
                .into_iter()
                .chain(segment_in.drain(coord.1 + 1..))
                .collect();
            if coord.0 == self.0.len() - 1 {
                vec![segment_out]
            } else {
                [segment_out]
                    .into_iter()
                    .chain(self.0.drain(coord.0 + 1..))
                    .collect()
            }
        };

        Self::new(result)
    }

    pub fn clone_part(
        &self,
        mut begin: SegmentCoordinate,
        mut end: SegmentCoordinate,
    ) -> Self {
        begin = self.get_closest_valid_coord(begin);
        end = self.get_closest_valid_coord(end);

        if begin.0 == end.0 {
            return if begin.1 == end.1 {
                Self::new(vec![])
            } else {
                Self::new(vec![self.0[begin.0]
                    .get(begin.1..end.1)
                    .unwrap()
                    .into()])
            };
        }

        let mut result = Vec::new();
        result.reserve(end.0 - begin.0 + 1);
        result.push(self.0[begin.0].get(begin.1..).unwrap().into());
        for i in (begin.0 + 1)..end.0 {
            result.push(self.0[i].clone());
        }

        if end.1 != 0 {
            result.push(self.0[end.0].get(0..end.1).unwrap().into());
        }
        Self::new(result)
    }

    pub fn get<'a>(&'a self, coord: SegmentCoordinate) -> Option<&'a Waypoint> {
        self.0.get(coord.0).and_then(|s| s.get(coord.1))
    }

    pub fn get_mut<'a>(
        &'a mut self,
        coord: SegmentCoordinate,
    ) -> Option<&'a mut Waypoint> {
        self.0.get_mut(coord.0).and_then(|s| s.get_mut(coord.1))
    }

    pub fn begin_coord(&self) -> SegmentCoordinate {
        (0, 0)
    }

    pub fn end_coord(&self) -> SegmentCoordinate {
        (self.0.len(), 0)
    }

    pub fn iter_between(
        &self,
        mut begin: SegmentCoordinate,
        mut end: SegmentCoordinate,
    ) -> SegmentsIterator<'_> {
        begin = self.get_closest_valid_coord(begin);
        end = self.get_closest_valid_coord(end);
        if end < begin {
            panic!("Begin {:?} must not be after end {:?}", begin, end);
        }

        SegmentsIterator {
            obj: &self,
            begin,
            end,
        }
    }

    pub fn iter(&self) -> SegmentsIterator<'_> {
        self.iter_between(self.begin_coord(), self.end_coord())
    }

    pub fn iter_from(&self, coord: SegmentCoordinate) -> SegmentsIterator<'_> {
        self.iter_between(coord, self.end_coord())
    }

    pub fn iter_until(&self, coord: SegmentCoordinate) -> SegmentsIterator<'_> {
        self.iter_between(self.begin_coord(), coord)
    }

    pub fn next_coord(&self, coord: SegmentCoordinate) -> SegmentCoordinate {
        match self.0.get(coord.0) {
            None => coord,
            Some(s) => {
                if coord.1 < s.len() - 1 {
                    (coord.0, coord.1 + 1)
                } else {
                    (coord.0 + 1, 0)
                }
            }
        }
    }

    pub fn prev_coord(&self, coord: SegmentCoordinate) -> SegmentCoordinate {
        if coord.1 == 0 {
            if coord.0 == 0 {
                coord
            } else {
                let prev = coord.0 - 1;
                match self.0.get(prev) {
                    None => coord,
                    Some(s) => (prev, s.len() - 1),
                }
            }
        } else {
            (coord.0, coord.1 - 1)
        }
    }

    fn get_closest_valid_coord(
        &self,
        coord: SegmentCoordinate,
    ) -> SegmentCoordinate {
        match self.0.get(coord.0) {
            None => self.end_coord(),
            Some(s) => {
                if coord.1 >= s.len() {
                    (coord.0 + 1, 0)
                } else {
                    coord
                }
            }
        }
    }

    pub fn process<F>(self, mut func: F) -> Result<Segments>
    where
        F: FnMut(
            &mut Segments,
            &mut Segment,
            &Waypoint,
            SegmentCoordinate,
        ) -> Result<()>,
    {
        let mut current_route: Segments = Segments::default();
        for segment in self.0 {
            let mut route_segment: Segment = Vec::new();
            for point in segment {
                let coordinate = (current_route.0.len(), route_segment.len());
                func(
                    &mut current_route,
                    &mut route_segment,
                    &point,
                    coordinate,
                )?;
                route_segment.push(point);
            }
            current_route.0.push(route_segment);
        }

        Ok(current_route)
    }

    pub fn commit<F, Ret>(
        &mut self,
        mut route_segment: Option<&mut Segment>,
        func: F,
    ) -> Vec<Activity>
    where
        F: FnOnce(&Segments) -> Ret,
        Ret: DoubleEndedIterator<Item = (ActivityType, SegmentCoordinate)>,
    {
        let mut result = Vec::new();
        let is_new_segment =
            route_segment.as_ref().map_or(true, |s| s.is_empty());

        if !is_new_segment {
            self.0.push(take(route_segment.as_mut().unwrap()));
        }
        let mut to_add: Vec<Activity> = func(&self)
            .rev()
            .map(|(t, c)| Activity::new(t, self.split_end(c)))
            .collect();

        if !self.0.is_empty() {
            to_add.push(Activity::new(ActivityType::default(), take(self)));
        }
        result.reserve(to_add.len());
        to_add.into_iter().rev().for_each(|r| result.push(r));
        if !is_new_segment {
            route_segment.unwrap().push(
                result
                    .last()
                    .unwrap()
                    .route
                    .0
                    .last()
                    .unwrap()
                    .last()
                    .unwrap()
                    .clone(),
            );
        }

        result
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

impl<'a> IntoIterator for &'a Segments {
    type Item = (SegmentCoordinate, &'a Waypoint);
    type IntoIter = SegmentsIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct SegmentsIterator<'a> {
    obj: &'a Segments,
    begin: SegmentCoordinate,
    end: SegmentCoordinate,
}

impl<'a> Iterator for SegmentsIterator<'a> {
    type Item = (SegmentCoordinate, &'a Waypoint);

    fn next(&mut self) -> Option<Self::Item> {
        if self.begin == self.end {
            return None;
        }

        let result = (self.begin, self.obj.get(self.begin).unwrap());
        self.begin = self.obj.next_coord(self.begin);
        Some(result)
    }
}

impl<'a> DoubleEndedIterator for SegmentsIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.begin == self.end {
            return None;
        }

        self.end = self.obj.prev_coord(self.end);
        Some((self.end, self.obj.get(self.end).unwrap()))
    }
}
