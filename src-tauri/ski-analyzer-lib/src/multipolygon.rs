use geo::{Contains, LineString, MultiPolygon, Polygon, Relate};
use topological_sort::TopologicalSort;

use super::osm_reader::{parse_way, Document, Relation};
use crate::error::{Error, ErrorType, Result};

use std::collections::HashMap;

type Line = Vec<u64>;

fn create_polygon(doc: &Document, line: &Line) -> Result<Polygon> {
    Ok(Polygon::new(
        LineString::new(parse_way(&doc, line)?),
        Vec::new(),
    ))
}

fn find_rings(doc: &Document, ways: Vec<Line>) -> Result<Vec<Polygon>> {
    let mut result = Vec::new();
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

    let mut endpoints: HashMap<u64, Vec<(usize, bool)>> = HashMap::new();

    let mut push = |id, i| endpoints.entry(id).or_default().push(i);

    for i in 0..lines.len() {
        push(*lines[i].first().unwrap(), (i, false));
        push(*lines[i].last().unwrap(), (i, true));
    }

    while let Some((key, value)) = endpoints.iter_mut().next() {
        if value.len() < 2 {
            return Err(Error::new(
                ErrorType::OSMError,
                format!(
                    "Unmatched endpoints in multipolygon: {:?}",
                    value
                        .iter()
                        .map(|i| &lines[i.0])
                        .collect::<Vec<&Vec<u64>>>()
                ),
            ));
        }
        let first = value.pop().unwrap();
        let second = value.pop().unwrap();
        if value.is_empty() {
            let key2 = *key;
            endpoints.remove(&key2);
        }

        let reverse = first.1 == second.1;
        if reverse {
            lines[first.0].reverse();
        }
        let (idx1, idx2) = if second.1 {
            (second.0, first.0)
        } else {
            (first.0, second.0)
        };

        let mut tail: Line = Line::new();
        tail.append(&mut lines[idx2]);
        let head = &mut lines[idx1];
        assert_eq!(head.last(), tail.first());
        head.pop();
        head.append(&mut tail);
        let first_id = *head.first().unwrap();
        let last_id = *head.last().unwrap();
        if first_id == last_id {
            result.push(create_polygon(&doc, &head)?);
            if {
                let v = endpoints.get_mut(&first_id).unwrap();
                v.retain(|x| x.0 != idx1 && x.0 != idx2);
                v.is_empty()
            } {
                endpoints.remove(&first_id);
            }
            head.clear();
        } else {
            let mut replace = |id, idx, value| {
                for i in endpoints.get_mut(id).unwrap().iter_mut() {
                    if i.0 == idx {
                        *i = value;
                        break;
                    }
                }
            };
            replace(&last_id, idx2, (idx1, true));
            if reverse {
                replace(&first_id, idx1, (idx1, false));
            }
        }
    }

    Ok(result)
}

fn sort_outer_polygons(mut input: Vec<Polygon>) -> Vec<Polygon> {
    let mut ordering: TopologicalSort<usize> = TopologicalSort::new();
    for i in 0..input.len() {
        ordering.insert(i);
    }

    for i in 0..(input.len() - 1) {
        for j in (i + 1)..input.len() {
            let int = input[i].relate(&input[j]);
            if int.matches("TFFT*F***").unwrap() {
                ordering.add_dependency(i, j);
            } else if int.matches("TT*F**FF*").unwrap() {
                ordering.add_dependency(j, i);
            }
        }
    }

    ordering
        .map(|i| {
            std::mem::replace(
                &mut input[i],
                Polygon::new(LineString::new(vec![]), vec![]),
            )
        })
        .collect()
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

    let mut outers = sort_outer_polygons(find_rings(&doc, outer_ways)?);
    let inners = find_rings(&doc, inner_ways)?;
    let mut remaining = inners.len();

    for inner in inners {
        for outer in &mut outers {
            if outer.contains(&inner) {
                outer.interiors_push(inner.into_inner().0);
                remaining -= 1;
                break;
            }
        }
    }

    if remaining != 0 {
        Err(Error::new(
            ErrorType::OSMError,
            format!("Multipolygon has {} orphaned inner rings.", remaining),
        ))
    } else {
        Ok(MultiPolygon(outers))
    }
}
