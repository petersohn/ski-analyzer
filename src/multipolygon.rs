use geo::{LineString, MultiPolygon, Polygon};

use super::osm_reader::{parse_way, Document, Relation, Way};
use crate::error::{Error, ErrorType, Result};

use std::cmp::{max, min};

type Line = Vec<u64>;

fn find_ring(lines: &mut Vec<Line>) -> Option<Line> {
    for i in 0..lines.len() {
        for j in 0..lines.len() {
            if i == j {
                continue;
            }

            if lines[i].last().unwrap() == lines[j].first().unwrap() {
                let mut ret = Line::new();
                ret.append(&mut lines[i]);
                ret.pop();
                ret.append(&mut lines[j]);
                lines.remove(max(i, j));
                lines.remove(min(i, j));
                return Some(ret);
            }
        }
    }
    None
}

fn create_polygon(doc: &Document, line: &Line) -> Result<Polygon> {
    Ok(Polygon::new(
        LineString::new(parse_way(&doc, line)?),
        Vec::new(),
    ))
}

fn find_rings(doc: &Document, ways: Vec<Line>) -> Result<Vec<Polygon>> {
    let mut result: Vec<Polygon> = Vec::new();
    let mut lines: Vec<Line> = Vec::new();
    for way in ways {
        if way.len() < 2 {
            return Err(Error::new_s(
                ErrorType::OSMError,
                "Way has less than 2 nodes in multipolygon",
            ));
        }
        if way.first().unwrap() == way.last().unwrap() {
            result.push(create_polygon(&doc, &way)?);
        } else {
            lines.push(way);
        }
    }

    while let Some(line) = find_ring(&mut lines) {
        result.push(create_polygon(&doc, &line)?);
    }

    if !lines.is_empty() {
        return Err(Error::new_s(
            ErrorType::OSMError,
            "Not all multipolygon ways are closed",
        ));
    }

    Ok(result)
}

pub fn parse_multipolygon(
    doc: &Document,
    input: &Relation,
) -> Result<MultiPolygon> {
    let mut outer_ways: Vec<Line> = Vec::new();
    let mut inner_ways: Vec<Line> = Vec::new();
    for member in &input.members.ways {
        let way = doc.elements.get_way(&member.ref_)?;
        match member.role.as_str() {
            "outer" => outer_ways.push(way.nodes.clone()),
            "inner" => inner_ways.push(way.nodes.clone()),
            _ => {
                return Err(Error::new(
                    ErrorType::OSMError,
                    format!("Invalid role for multipolygon: {}", member.role),
                ));
            }
        };
    }

    let outers = find_rings(&doc, outer_ways)?;
    let inners = find_rings(&doc, inner_ways)?;

    Err(Error::new_s(ErrorType::LogicError, "x"))
}
