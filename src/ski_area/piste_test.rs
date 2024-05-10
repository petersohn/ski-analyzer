use super::piste::parse_pistes;
use super::{Difficulty, Piste, PisteMetadata};
use crate::config::{set_config, Config};
use crate::osm_reader::{Document, Node, Tags, Way};
use geo::{Coord, LineString, MultiLineString, MultiPolygon, Polygon};
use std::collections::HashSet;
use std::hash::Hash;

use rstest::{fixture, rstest};

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

    fn add_node(&mut self, p: Point) -> u64 {
        let id = self.get_id();
        self.document.elements.nodes.insert(
            id,
            Node {
                lat: i2f(p.1),
                lon: i2f(p.0),
                tags: Tags::new(),
            },
        );
        id
    }

    fn add_way(&mut self, line: Line, tags: &TagsDef) {
        let mut nodes: Vec<u64> = Vec::new();
        nodes.reserve(line.len());
        for p in line {
            nodes.push(self.add_node(p));
        }

        let id = self.get_id();
        self.document.elements.ways.insert(
            id,
            Way {
                nodes,
                tags: to_tags(&tags),
            },
        );
    }
}

fn create_document(ways: Vec<WayDef>) -> Document {
    let mut builder = DocumentBuilder::new();
    for way in ways {
        builder.add_way(way.line, &way.tags);
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

fn to_line_a(area: &Polygon) -> Line {
    to_line(area.exterior())
}

fn to_lines_a(areas: &MultiPolygon) -> Vec<Line> {
    let mut result: Vec<Line> = areas.iter().map(to_line_a).collect();
    result.sort();
    result
}

fn to_set<T>(vec: Vec<T>) -> HashSet<T>
where
    T: Eq + Hash,
{
    vec.into_iter().collect()
}

#[derive(PartialEq, Eq, Debug, Hash)]
struct PisteOut {
    metadata: PisteMetadata,
    areas: Vec<Line>,
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

    fn list(pistes: &Vec<Piste>) -> Vec<PisteOut> {
        pistes.iter().map(PisteOut::new).collect()
    }
}

struct Init;

#[fixture]
fn init() -> Init {
    match set_config(Config { verbose: 2 }) {
        Ok(()) => (),
        Err(_) => (),
    }
    Init {}
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

    let pistes = parse_pistes(&document);

    assert_eq!(pistes.len(), 1);
    assert_eq!(pistes[0].metadata.name, "Piste 1");
    assert_eq!(pistes[0].metadata.ref_, "a");
    assert_eq!(pistes[0].metadata.difficulty, Difficulty::Advanced);
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

    let pistes = parse_pistes(&document);

    assert_eq!(pistes.len(), 1);
    assert_eq!(pistes[0].metadata.name, "Piste 1");
    assert_eq!(pistes[0].metadata.ref_, "a");
    assert_eq!(pistes[0].metadata.difficulty, Difficulty::Unknown);
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

    let pistes = parse_pistes(&document);

    assert_eq!(pistes.len(), 1);
    assert_eq!(pistes[0].metadata.name, "");
    assert_eq!(pistes[0].metadata.ref_, "b");
    assert_eq!(pistes[0].metadata.difficulty, Difficulty::Novice);
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

    let pistes = parse_pistes(&document);

    assert_eq!(pistes.len(), 1);
    assert_eq!(pistes[0].metadata.name, "Some Name");
    assert_eq!(pistes[0].metadata.ref_, "");
    assert_eq!(pistes[0].metadata.difficulty, Difficulty::Intermediate);
}

#[rstest]
fn metadata_no_name_or_ref(_init: Init, line0: Line) {
    let document = create_document(vec![WayDef {
        line: line0,
        tags: vec![("piste:type", "downhill"), ("piste:difficulty", "novice")],
    }]);

    let pistes = parse_pistes(&document);

    assert_eq!(pistes.len(), 1);
    assert_eq!(pistes[0].metadata.name, "");
    assert_eq!(pistes[0].metadata.ref_, "");
    assert_eq!(pistes[0].metadata.difficulty, Difficulty::Novice);
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

    let pistes = parse_pistes(&document);

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

    let pistes = parse_pistes(&document);

    assert_eq!(pistes.len(), 1);
    assert_eq!(pistes[0].metadata.name, "Good Name");
    assert_eq!(pistes[0].metadata.ref_, "b");
    assert_eq!(pistes[0].metadata.difficulty, Difficulty::Easy);
}

#[rstest]
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

    let pistes = parse_pistes(&document);
    let expected = vec![PisteOut {
        metadata: PisteMetadata {
            ref_: "1".to_owned(),
            name: "Piste 1".to_owned(),
            difficulty: Difficulty::Easy,
        },
        lines: vec![line0],
        areas: vec![area00, area01],
    }];
    let actual = PisteOut::list(&pistes);
    assert_eq!(actual, expected);
}

#[rstest]
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

    let pistes = parse_pistes(&document);
    let expected = vec![
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
            areas: vec![area00],
        },
    ];
    let actual = PisteOut::list(&pistes);
    assert_eq!(to_set(actual), to_set(expected));
}

#[rstest]
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

    let pistes = parse_pistes(&document);
    let expected = vec![
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
            areas: vec![area00],
        },
    ];
    let actual = PisteOut::list(&pistes);
    assert_eq!(to_set(actual), to_set(expected));
}
