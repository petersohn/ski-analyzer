use super::piste::parse_pistes;
use super::{Difficulty, Piste, PisteData, PisteMetadata, SkiArea};
use crate::osm_reader::{
    Coordinate, Document, Node, Relation, RelationMember, RelationMembers,
    Tags, Way,
};
use crate::utils::cancel::CancellationToken;
use crate::utils::rect::union_rects_if;
use crate::utils::test_util::{
    assert_eq_pretty, create_ski_area_metadata, init, save_ski_area, Init,
};

use ::function_name::named;
use geo::{
    BoundingRect, Coord, LineString, MultiLineString, MultiPolygon, Polygon,
    Rect,
};
use rstest::{fixture, rstest};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::Hash;
use time::OffsetDateTime;

type Point = (i64, i64);
type Line = Vec<Point>;
type TagsDef = Vec<(&'static str, &'static str)>;

fn to_tags(tags: &TagsDef) -> Tags {
    let mut result = Tags::new();
    for (key, value) in tags {
        result.insert(key.to_string(), value.to_string());
    }
    result
}

const RATIO: f64 = 10000000.0;

fn f2i(f: f64) -> i64 {
    (f * RATIO).round() as i64
}

fn i2f(i: i64) -> f64 {
    (i as f64) / RATIO
}

struct WayDef {
    line: Line,
    tags: TagsDef,
}

#[derive(Default)]
struct DocumentBuilder {
    id: u64,
    node_cache: HashMap<Point, u64>,
    document: Document,
}

impl DocumentBuilder {
    fn new() -> Self {
        DocumentBuilder::default()
    }

    fn get_id(&mut self) -> u64 {
        self.id += 1;
        self.id
    }

    fn add_node(&mut self, p: &Point) -> u64 {
        if let Some(id) = self.node_cache.get(p) {
            return *id;
        }

        let id = self.get_id();
        self.document.elements.nodes.insert(
            id,
            Node {
                coordinate: Coordinate {
                    lat: i2f(p.1),
                    lon: i2f(p.0),
                },
                tags: Tags::new(),
            },
        );
        self.node_cache.insert(*p, id);
        id
    }

    fn add_way(&mut self, line: &[Point], tags: &TagsDef) -> u64 {
        let nodes = line.into_iter().map(|p| self.add_node(p)).collect();

        let id = self.get_id();
        self.document.elements.ways.insert(
            id,
            Way {
                nodes,
                tags: to_tags(&tags),
                geometry: vec![],
            },
        );
        id
    }

    fn to_member(data: &(u64, &str)) -> RelationMember {
        RelationMember {
            ref_: data.0,
            role: data.1.to_string(),
        }
    }

    fn add_relation(
        &mut self,
        nodes: &[(u64, &str)],
        ways: &[(u64, &str)],
        tags: &TagsDef,
    ) -> u64 {
        let members = RelationMembers {
            nodes: nodes.iter().map(DocumentBuilder::to_member).collect(),
            ways: ways.iter().map(DocumentBuilder::to_member).collect(),
        };

        let id = self.get_id();
        self.document.elements.relations.insert(
            id,
            Relation {
                members,
                tags: to_tags(&tags),
            },
        );

        id
    }
}

fn create_document(ways: Vec<WayDef>) -> Document {
    let mut builder = DocumentBuilder::new();
    for way in ways {
        builder.add_way(&way.line, &way.tags);
    }
    builder.document
}

fn to_point(coord: &Coord) -> Point {
    (f2i(coord.x), f2i(coord.y))
}

fn to_line(line: &LineString) -> Line {
    line.0.iter().map(to_point).collect()
}

fn to_lines(lines: &MultiLineString) -> Vec<Line> {
    let mut result: Vec<Line> = lines.iter().map(to_line).collect();
    result.sort();
    result
}

fn to_line_a(area: &Polygon) -> Area {
    Area {
        exterior: to_line(area.exterior()),
        interiors: area.interiors().iter().map(to_line).collect(),
    }
}

fn to_lines_a(areas: &MultiPolygon) -> Vec<Area> {
    let mut result: Vec<Area> = areas.iter().map(to_line_a).collect();
    result.sort();
    result
}

fn to_set<T>(vec: Vec<T>) -> HashSet<T>
where
    T: Eq + Hash,
{
    vec.into_iter().collect()
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
struct Area {
    exterior: Line,
    interiors: Vec<Line>,
}

impl Area {
    fn simple(exterior: Line) -> Self {
        Area {
            exterior,
            interiors: Vec::new(),
        }
    }

    fn multi(exterior: Line, interiors: Vec<Line>) -> Self {
        Area {
            exterior,
            interiors,
        }
    }
}

fn to_coord(p: &Point) -> Coord {
    Coord {
        x: i2f(p.0),
        y: i2f(p.1),
    }
}

fn to_line_string(l: &Line) -> LineString {
    LineString::new(l.iter().map(to_coord).collect())
}

fn to_multi_line_string(lines: &[Line]) -> MultiLineString {
    MultiLineString::new(lines.iter().map(to_line_string).collect())
}

fn to_multi_polygon(areas: &[Area]) -> MultiPolygon {
    MultiPolygon::new(
        areas
            .iter()
            .map(|a| {
                Polygon::new(
                    to_line_string(&a.exterior),
                    a.interiors.iter().map(to_line_string).collect(),
                )
            })
            .collect(),
    )
}

#[derive(PartialEq, Eq, Debug, Hash)]
struct PisteOut {
    metadata: PisteMetadata,
    areas: Vec<Area>,
    lines: Vec<Line>,
}

impl PisteOut {
    fn new(piste: &Piste) -> Self {
        PisteOut {
            metadata: piste.metadata.clone(),
            areas: to_lines_a(&piste.data.areas),
            lines: to_lines(&piste.data.lines),
        }
    }

    fn list<'a, It>(pistes: It) -> Vec<PisteOut>
    where
        It: Iterator<Item = &'a Piste>,
    {
        pistes.map(PisteOut::new).collect()
    }

    fn to_piste(&self) -> Piste {
        let areas = to_multi_polygon(&self.areas);
        let lines = to_multi_line_string(&self.lines);
        let bounding_rect =
            union_rects_if(areas.bounding_rect(), lines.bounding_rect())
                .unwrap();
        Piste {
            // Should be unique enough for the tests
            metadata: self.metadata.clone(),
            data: PisteData {
                areas,
                lines,
                bounding_rect,
            },
        }
    }
}

fn to_ski_area<Pistes, T>(name: String, pistes_in: Pistes) -> SkiArea
where
    Pistes: Iterator<Item = T>,
    T: Borrow<Piste>,
{
    let mut pistes = HashMap::new();
    let mut bounding_rect: Option<Rect> = None;

    for p in pistes_in {
        let piste = p.borrow();
        pistes.insert(format!("{:?}", piste.metadata), piste.clone());
        bounding_rect =
            union_rects_if(bounding_rect, Some(piste.data.bounding_rect));
    }

    SkiArea::new(
        create_ski_area_metadata(name),
        HashMap::new(),
        pistes,
        OffsetDateTime::now_utc(),
    )
    .unwrap()
}

fn save_output<'a, PisteOuts, Pistes, T>(
    expected: PisteOuts,
    actual: Pistes,
    name: &str,
) where
    PisteOuts: Iterator<Item = &'a PisteOut>,
    Pistes: Iterator<Item = T>,
    T: Borrow<Piste>,
{
    let dir = format!("test_output/piste_test/{}", name);
    fs::create_dir_all(&dir).unwrap();
    save_ski_area(
        &to_ski_area(
            format!("{}.Expected", name),
            expected.map(|p| p.to_piste()),
        ),
        &format!("{}/expected.json", dir),
    );
    save_ski_area(
        &to_ski_area(format!("{}.Actual", name), actual),
        &format!("{}/actual.json", dir),
    );
}

#[fixture]
fn line0() -> Line {
    vec![
        (65212291, 453164462),
        (65214277, 453170269),
        (65217032, 453174316),
        (65219256, 453176102),
        (65221216, 453177862),
        (65225114, 453180105),
        (65231628, 453184813),
        (65236938, 453187682),
        (65241808, 453191968),
        (65244890, 453197656),
        (65247164, 453201727),
        (65250230, 453206450),
        (65251761, 453208964),
        (65251324, 453212819),
        (65251929, 453218024),
        (65258114, 453225458),
        (65269514, 453230572),
        (65275646, 453231524),
        (65285838, 453238372),
        (65294963, 453244548),
        (65300600, 453246514),
        (65304261, 453250384),
        (65305019, 453254556),
        (65305143, 453258076),
        (65306505, 453259256),
    ]
}

// line0[..LINE0_MIDPOINT] intersects area00, line0[LINE0_MIDPOINT..] intersects area01
const LINE0_MIDPOINT: usize = 12;

#[fixture]
fn area00() -> Line {
    vec![
        (65222413, 453161829),
        (65216064, 453167159),
        (65215938, 453170868),
        (65219128, 453174177),
        (65227845, 453178285),
        (65231729, 453182172),
        (65237446, 453184370),
        (65242468, 453187213),
        (65246415, 453192543),
        (65248026, 453198294),
        (65251027, 453203979),
        (65253206, 453205023),
        (65259554, 453204113),
        (65256358, 453208932),
        (65251826, 453208788),
        (65247900, 453208465),
        (65241425, 453203891),
        (65241489, 453198095),
        (65237730, 453190366),
        (65230561, 453185481),
        (65220391, 453179795),
        (65213917, 453173022),
        (65211611, 453168802),
        (65210158, 453165427),
        (65214454, 453164849),
        (65222413, 453161829),
    ]
}

#[fixture]
fn area01() -> Line {
    vec![
        (65255638, 453210864),
        (65254957, 453213342),
        (65255037, 453216475),
        (65256822, 453220399),
        (65259734, 453223770),
        (65265170, 453226009),
        (65268323, 453227151),
        (65268049, 453228542),
        (65269514, 453230572),
        (65272505, 453231437),
        (65275646, 453231524),
        (65280724, 453231053),
        (65285501, 453234287),
        (65289256, 453236735),
        (65298220, 453242307),
        (65308093, 453246234),
        (65306764, 453250300),
        (65305831, 453255242),
        (65305349, 453257990),
        (65306505, 453259256),
        (65303398, 453259218),
        (65301249, 453252752),
        (65302497, 453250120),
        (65283468, 453239373),
        (65279437, 453240041),
        (65275207, 453236911),
        (65271330, 453233552),
        (65262581, 453229447),
        (65256983, 453226676),
        (65252061, 453221610),
        (65249810, 453216905),
        (65249375, 453213150),
        (65249574, 453209220),
        (65251761, 453208964),
        (65255638, 453210864),
    ]
}

#[fixture]
fn line1() -> Line {
    vec![
        (65303747, 453196734),
        (65304339, 453202588),
        (65303208, 453204805),
        (65303397, 453205894),
        (65302758, 453206639),
        (65301197, 453207041),
        (65296050, 453207657),
        (65291501, 453210284),
        (65278849, 453221287),
        (65269053, 453227548),
        (65269514, 453230572),
        (65270658, 453231341),
        (65273029, 453231763),
        (65275646, 453231524),
        (65280598, 453230043),
        (65286126, 453224605),
        (65291243, 453220060),
        (65296630, 453218640),
        (65310097, 453217314),
        (65319928, 453211443),
        (65328143, 453205667),
        (65336532, 453197864),
        (65344572, 453191747),
        (65353989, 453189747),
    ]
}

#[fixture]
fn area10() -> Line {
    vec![
        (65307853, 453202675),
        (65307246, 453203286),
        (65302674, 453207889),
        (65298072, 453211301),
        (65297386, 453215220),
        (65296942, 453217890),
        (65304135, 453217580),
        (65310778, 453216418),
        (65321086, 453209964),
        (65327094, 453205294),
        (65335942, 453197697),
        (65343907, 453191417),
        (65350165, 453189883),
        (65350800, 453190529),
        (65348423, 453191520),
        (65344624, 453193104),
        (65334205, 453200526),
        (65325193, 453208879),
        (65311605, 453217813),
        (65294792, 453219732),
        (65291264, 453220817),
        (65287157, 453224383),
        (65280724, 453231053),
        (65275646, 453231524),
        (65272505, 453231437),
        (65269514, 453230572),
        (65268049, 453228542),
        (65268323, 453227151),
        (65271419, 453224247),
        (65278944, 453219499),
        (65282830, 453215119),
        (65286385, 453211592),
        (65292394, 453208491),
        (65297217, 453206767),
        (65300084, 453206612),
        (65296225, 453204712),
        (65291295, 453202203),
        (65298653, 453198428),
        (65302131, 453198196),
        (65302543, 453200982),
        (65305151, 453202559),
        (65307853, 453202675),
    ]
}

#[fixture]
fn area11() -> Line {
    vec![
        (65292294, 453218060),
        (65289396, 453213190),
        (65279254, 453222061),
        (65273741, 453225540),
        (65284131, 453225490),
        (65290280, 453219775),
        (65292294, 453218060),
    ]
}

#[rstest]
fn metadta_basic(_init: Init, line0: Line) {
    let document = create_document(vec![WayDef {
        line: line0,
        tags: vec![
            ("piste:type", "downhill"),
            ("piste:difficulty", "advanced"),
            ("name", "Piste 1"),
            ("ref", "a"),
        ],
    }]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();

    assert_eq!(pistes.len(), 1);
    let piste = pistes.iter().next().unwrap().1;
    assert_eq!(piste.metadata.name, "Piste 1");
    assert_eq!(piste.metadata.ref_, "a");
    assert_eq!(piste.metadata.difficulty, Difficulty::Advanced);
}

#[rstest]
fn metadata_no_difficulty(_init: Init, line0: Line) {
    let document = create_document(vec![WayDef {
        line: line0,
        tags: vec![
            ("piste:type", "downhill"),
            ("name", "Piste 1"),
            ("ref", "a"),
        ],
    }]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();

    assert_eq!(pistes.len(), 1);
    let piste = pistes.iter().next().unwrap().1;
    assert_eq!(piste.metadata.name, "Piste 1");
    assert_eq!(piste.metadata.ref_, "a");
    assert_eq!(piste.metadata.difficulty, Difficulty::Unknown);
}

#[rstest]
fn metadata_no_name(_init: Init, line0: Line) {
    let document = create_document(vec![WayDef {
        line: line0,
        tags: vec![
            ("piste:type", "downhill"),
            ("piste:difficulty", "novice"),
            ("ref", "b"),
        ],
    }]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();

    assert_eq!(pistes.len(), 1);
    let piste = pistes.iter().next().unwrap().1;
    assert_eq!(piste.metadata.name, "");
    assert_eq!(piste.metadata.ref_, "b");
    assert_eq!(piste.metadata.difficulty, Difficulty::Novice);
}

#[rstest]
fn metadata_no_ref(_init: Init, line0: Line) {
    let document = create_document(vec![WayDef {
        line: line0,
        tags: vec![
            ("piste:type", "downhill"),
            ("piste:difficulty", "intermediate"),
            ("name", "Some Name"),
        ],
    }]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();

    assert_eq!(pistes.len(), 1);
    let piste = pistes.iter().next().unwrap().1;
    assert_eq!(piste.metadata.name, "Some Name");
    assert_eq!(piste.metadata.ref_, "");
    assert_eq!(piste.metadata.difficulty, Difficulty::Intermediate);
}

#[rstest]
fn metadata_no_name_or_ref(_init: Init, line0: Line) {
    let document = create_document(vec![WayDef {
        line: line0,
        tags: vec![("piste:type", "downhill"), ("piste:difficulty", "novice")],
    }]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();

    assert_eq!(pistes.len(), 1);
    let piste = pistes.iter().next().unwrap().1;
    assert_eq!(piste.metadata.name, "");
    assert_eq!(piste.metadata.ref_, "");
    assert_eq!(piste.metadata.difficulty, Difficulty::Novice);
}

#[rstest]
fn metadata_bad_type(_init: Init, line0: Line) {
    let document = create_document(vec![WayDef {
        line: line0,
        tags: vec![
            ("piste:type", "foobar"),
            ("piste:difficulty", "advanced"),
            ("name", "Piste 1"),
            ("ref", "a"),
        ],
    }]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();

    assert_eq!(pistes.len(), 0);
}

#[rstest]
fn metadata_alternate_naming(_init: Init, line0: Line) {
    let document = create_document(vec![WayDef {
        line: line0,
        tags: vec![
            ("piste:type", "downhill"),
            ("piste:difficulty", "easy"),
            ("name", "Bad Name"),
            ("piste:name", "Good Name"),
            ("ref", "a"),
            ("piste:ref", "b"),
        ],
    }]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();

    assert_eq!(pistes.len(), 1);
    let piste = pistes.iter().next().unwrap().1;
    assert_eq!(piste.metadata.name, "Good Name");
    assert_eq!(piste.metadata.ref_, "b");
    assert_eq!(piste.metadata.difficulty, Difficulty::Easy);
}

#[rstest]
#[named]
fn find_areas_to_line(_init: Init, line0: Line, area00: Line, area01: Line) {
    let document = create_document(vec![
        WayDef {
            line: line0.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "easy"),
                ("ref", "1"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: area00.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "easy"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: area01.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "easy"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = vec![PisteOut {
        metadata: PisteMetadata {
            ref_: "1".to_owned(),
            name: "Piste 1".to_owned(),
            difficulty: Difficulty::Easy,
        },
        lines: vec![line0],
        areas: vec![Area::simple(area00), Area::simple(area01)],
    }];
    let actual = PisteOut::list(pistes.values());
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn find_lines_to_area(_init: Init, line0: Line, area00: Line) {
    let line00 = line0[0..5].to_vec();
    let line01 = line0[5..LINE0_MIDPOINT].to_vec();
    let document = create_document(vec![
        WayDef {
            line: line00.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "advanced"),
            ],
        },
        WayDef {
            line: line01.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "advanced"),
                ("ref", "1"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: area00.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "advanced"),
                ("name", "Piste 1"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = vec![PisteOut {
        metadata: PisteMetadata {
            ref_: "1".to_owned(),
            name: "Piste 1".to_owned(),
            difficulty: Difficulty::Advanced,
        },
        lines: vec![line00, line01],
        areas: vec![Area::simple(area00)],
    }];
    let actual = PisteOut::list(pistes.values());
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn merge_unnamed_pistes(_init: Init, line0: Line, area00: Line, area01: Line) {
    let line00 = line0[0..8].to_vec();
    let line01 = line0[8..].to_vec();
    let document = create_document(vec![
        WayDef {
            line: line00.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: line01.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
            ],
        },
        WayDef {
            line: area00.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
            ],
        },
        WayDef {
            line: area01.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = vec![PisteOut {
        metadata: PisteMetadata {
            ref_: String::new(),
            name: "Piste 1".to_owned(),
            difficulty: Difficulty::Intermediate,
        },
        lines: vec![line00, line01],
        areas: vec![Area::simple(area00), Area::simple(area01)],
    }];
    let actual = PisteOut::list(pistes.values());
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn orphaned_unnamed_area(_init: Init, line0: Line, area01: Line) {
    let line00 = line0[0..LINE0_MIDPOINT].to_vec();
    let document = create_document(vec![
        WayDef {
            line: line00.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
                ("ref", "1a"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: area01.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = to_set(vec![
        PisteOut {
            metadata: PisteMetadata {
                ref_: "1a".to_owned(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Intermediate,
            },
            lines: vec![line00],
            areas: vec![],
        },
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: String::new(),
                difficulty: Difficulty::Intermediate,
            },
            lines: vec![],
            areas: vec![Area::simple(area01)],
        },
    ]);
    let actual = to_set(PisteOut::list(pistes.values()));
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn orphaned_unnamed_line(_init: Init, line0: Line, area01: Line) {
    let line00 = line0[0..LINE0_MIDPOINT].to_vec();
    let document = create_document(vec![
        WayDef {
            line: line00.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "novice"),
            ],
        },
        WayDef {
            line: area01.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "novice"),
                ("ref", "1a"),
                ("name", "Piste 1"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = to_set(vec![
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: String::new(),
                difficulty: Difficulty::Novice,
            },
            lines: vec![line00],
            areas: vec![],
        },
        PisteOut {
            metadata: PisteMetadata {
                ref_: "1a".to_owned(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Novice,
            },
            lines: vec![],
            areas: vec![Area::simple(area01)],
        },
    ]);
    let actual = to_set(PisteOut::list(pistes.values()));
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn merge_unnamed_line_and_area(
    _init: Init,
    line0: Line,
    area00: Line,
    area01: Line,
) {
    let line00 = line0[..LINE0_MIDPOINT].to_vec();
    let line01 = line0[LINE0_MIDPOINT..].to_vec();
    let document = create_document(vec![
        WayDef {
            line: line00.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
            ],
        },
        WayDef {
            line: line01.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
            ],
        },
        WayDef {
            line: area00.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
            ],
        },
        WayDef {
            line: area01.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
                ("name", "Piste 1"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = to_set(vec![
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: String::new(),
                difficulty: Difficulty::Intermediate,
            },
            lines: vec![line00],
            areas: vec![Area::simple(area00)],
        },
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Intermediate,
            },
            lines: vec![line01],
            areas: vec![Area::simple(area01)],
        },
    ]);
    let actual = to_set(PisteOut::list(pistes.values()));
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn different_difficulty(_init: Init, line0: Line, area00: Line) {
    let document = create_document(vec![
        WayDef {
            line: line0.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "easy"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: area00.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
                ("name", "Piste 1"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = to_set(vec![
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Easy,
            },
            lines: vec![line0],
            areas: vec![],
        },
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Intermediate,
            },
            lines: vec![],
            areas: vec![Area::simple(area00)],
        },
    ]);
    let actual = to_set(PisteOut::list(pistes.values()));
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn different_name(_init: Init, line0: Line, area00: Line) {
    let document = create_document(vec![
        WayDef {
            line: line0.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: area00.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
                ("name", "Piste 2"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = to_set(vec![
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Intermediate,
            },
            lines: vec![line0],
            areas: vec![],
        },
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 2".to_owned(),
                difficulty: Difficulty::Intermediate,
            },
            lines: vec![],
            areas: vec![Area::simple(area00)],
        },
    ]);
    let actual = to_set(PisteOut::list(pistes.values()));
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn not_intersecting_line_and_area(_init: Init, line0: Line, area00: Line) {
    let line00 = line0[LINE0_MIDPOINT..].to_vec();
    let document = create_document(vec![
        WayDef {
            line: line00.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "advanced"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: area00.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "advanced"),
                ("name", "Piste 1"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = to_set(vec![
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Advanced,
            },
            lines: vec![line00],
            areas: vec![],
        },
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Advanced,
            },
            lines: vec![],
            areas: vec![Area::simple(area00)],
        },
    ]);
    let actual = to_set(PisteOut::list(pistes.values()));
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn use_larger_overlap(_init: Init, line0: Line, area00: Line, area01: Line) {
    let splitpoint = LINE0_MIDPOINT - 1;
    let line00 = line0[..splitpoint].to_vec();
    let line01 = line0[splitpoint..].to_vec();
    let document = create_document(vec![
        WayDef {
            line: line00.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
            ],
        },
        WayDef {
            line: line01.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
            ],
        },
        WayDef {
            line: area00.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: area01.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "intermediate"),
                ("name", "Piste 2"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = to_set(vec![
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Intermediate,
            },
            lines: vec![line00],
            areas: vec![Area::simple(area00)],
        },
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 2".to_owned(),
                difficulty: Difficulty::Intermediate,
            },
            lines: vec![line01],
            areas: vec![Area::simple(area01)],
        },
    ]);
    let actual = to_set(PisteOut::list(pistes.values()));
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn multipolygon_piste(_init: Init, line1: Line, area10: Line, area11: Line) {
    let mut builder = DocumentBuilder::new();
    builder.add_way(
        &line1,
        &vec![
            ("piste:type", "downhill"),
            ("piste:difficulty", "intermediate"),
            ("name", "Piste 1"),
        ],
    );
    let outer = builder.add_way(&area10, &vec![]);
    let inner = builder.add_way(&area11, &vec![]);
    builder.add_relation(
        &[],
        &[(inner, "inner"), (outer, "outer")],
        &vec![
            ("type", "multipolygon"),
            ("piste:type", "downhill"),
            ("piste:difficulty", "intermediate"),
            ("name", "Piste 1"),
        ],
    );
    let document = builder.document;

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = to_set(vec![PisteOut {
        metadata: PisteMetadata {
            ref_: String::new(),
            name: "Piste 1".to_owned(),
            difficulty: Difficulty::Intermediate,
        },
        lines: vec![line1],
        areas: vec![Area::multi(area10, vec![area11])],
    }]);
    let actual = to_set(PisteOut::list(pistes.values()));
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn multiple_polygons_in_multipolygon(
    _init: Init,
    area00: Line,
    area10: Line,
    area11: Line,
) {
    let mut builder = DocumentBuilder::new();
    let outer0 = builder.add_way(&area00, &vec![]);
    let outer1 = builder.add_way(&area10, &vec![]);
    let inner1 = builder.add_way(&area11, &vec![]);
    builder.add_relation(
        &[],
        &[(outer0, "outer"), (inner1, "inner"), (outer1, "outer")],
        &vec![
            ("type", "multipolygon"),
            ("piste:type", "downhill"),
            ("piste:difficulty", "easy"),
            ("name", "Piste 1"),
        ],
    );
    let document = builder.document;

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = to_set(vec![
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Easy,
            },
            lines: vec![],
            areas: vec![Area::simple(area00)],
        },
        PisteOut {
            metadata: PisteMetadata {
                ref_: String::new(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Easy,
            },
            lines: vec![],
            areas: vec![Area::multi(area10, vec![area11])],
        },
    ]);
    let actual = to_set(PisteOut::list(pistes.values()));
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn metadata_from_route(_init: Init, line0: Line, line1: Line) {
    let mut builder = DocumentBuilder::new();
    let l0 = builder.add_way(
        &line0,
        &vec![
            ("piste:type", "downhill"),
            ("piste:difficulty", "intermediate"),
            ("name", "Piste 1d"),
            ("ref", "1d"),
        ],
    );
    let l1 = builder.add_way(&line1, &vec![("piste:type", "downhill")]);
    builder.add_relation(
        &[],
        &[(l0, ""), (l1, "")],
        &vec![
            ("type", "route"),
            ("route", "piste"),
            ("piste:type", "downhill"),
            ("piste:difficulty", "easy"),
            ("name", "Piste 1"),
            ("ref", "1"),
        ],
    );
    let document = builder.document;

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = to_set(vec![
        PisteOut {
            metadata: PisteMetadata {
                ref_: "1".to_owned(),
                name: "Piste 1".to_owned(),
                difficulty: Difficulty::Easy,
            },
            lines: vec![line1],
            areas: vec![],
        },
        PisteOut {
            metadata: PisteMetadata {
                ref_: "1d".to_owned(),
                name: "Piste 1d".to_owned(),
                difficulty: Difficulty::Intermediate,
            },
            lines: vec![line0],
            areas: vec![],
        },
    ]);
    let actual = to_set(PisteOut::list(pistes.values()));
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn closed_line_is_area(_init: Init, area00: Line) {
    let document = create_document(vec![WayDef {
        line: area00.clone(),
        tags: vec![
            ("piste:type", "downhill"),
            ("piste:difficulty", "easy"),
            ("ref", "1"),
            ("name", "Piste 1"),
        ],
    }]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = vec![PisteOut {
        metadata: PisteMetadata {
            ref_: "1".to_owned(),
            name: "Piste 1".to_owned(),
            difficulty: Difficulty::Easy,
        },
        lines: vec![],
        areas: vec![Area::simple(area00)],
    }];
    let actual = PisteOut::list(pistes.values());
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn explicit_area(_init: Init, line0: Line) {
    let document = create_document(vec![WayDef {
        line: line0.clone(),
        tags: vec![
            ("piste:type", "downhill"),
            ("piste:difficulty", "easy"),
            ("ref", "1"),
            ("name", "Piste 1"),
            ("area", "yes"),
        ],
    }]);

    let mut expected_area = line0.clone();
    expected_area.push(expected_area[0]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = vec![PisteOut {
        metadata: PisteMetadata {
            ref_: "1".to_owned(),
            name: "Piste 1".to_owned(),
            difficulty: Difficulty::Easy,
        },
        lines: vec![],
        areas: vec![Area::simple(expected_area)],
    }];
    let actual = PisteOut::list(pistes.values());
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}

#[rstest]
#[named]
fn find_refs_complex(_init: Init, line0: Line, area00: Line, area01: Line) {
    let line00 = line0[0..5].to_vec();
    let line01 = line0[0..8].to_vec();
    let line02 = line0[8..].to_vec();
    let document = create_document(vec![
        WayDef {
            line: line00.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "advanced"),
                ("ref", "1"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: line01.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "advanced"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: line02.clone(),
            tags: vec![
                ("piste:type", "downhill"),
                ("piste:difficulty", "advanced"),
                ("ref", "1"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: area00.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "advanced"),
                ("name", "Piste 1"),
            ],
        },
        WayDef {
            line: area01.clone(),
            tags: vec![
                ("area", "yes"),
                ("piste:type", "downhill"),
                ("piste:difficulty", "advanced"),
                ("name", "Piste 1"),
            ],
        },
    ]);

    let pistes = parse_pistes(&CancellationToken::new(), &document).unwrap();
    let expected = vec![PisteOut {
        metadata: PisteMetadata {
            ref_: "1".to_owned(),
            name: "Piste 1".to_owned(),
            difficulty: Difficulty::Advanced,
        },
        lines: vec![line00, line01, line02],
        areas: vec![Area::simple(area00), Area::simple(area01)],
    }];
    let actual = PisteOut::list(pistes.values());
    save_output(expected.iter(), pistes.values(), function_name!());
    assert_eq_pretty!(actual, expected);
}
