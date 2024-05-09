use super::piste::parse_pistes;
use super::{Difficulty, Piste, PisteMetadata};
use crate::config::{set_config, Config};
use crate::osm_reader::{Document, Node, Tags, Way};
use geo::{Coord, LineString, MultiLineString, MultiPolygon, Polygon};

use rstest::{fixture, rstest};

type Point = (f64, f64);
type Line = Vec<Point>;
type TagsDef = Vec<(&'static str, &'static str)>;

fn to_tags(tags: &TagsDef) -> Tags {
    let mut result = Tags::new();
    for (key, value) in tags {
        result.insert(key.to_string(), value.to_string());
    }
    result
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
                lat: p.1,
                lon: p.0,
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
    coord.x_y()
}

fn to_line(line: &LineString) -> Line {
    line.0.iter().map(to_point).collect()
}

fn to_lines(lines: &MultiLineString) -> Vec<Line> {
    lines.iter().map(to_line).collect()
}

fn to_line_a(area: &Polygon) -> Line {
    to_line(area.exterior())
}

fn to_lines_a(areas: &MultiPolygon) -> Vec<Line> {
    areas.iter().map(to_line_a).collect()
}

#[derive(PartialEq, Debug)]
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
        (6.5212291, 45.3164462),
        (6.5214277, 45.3170269),
        (6.5217032, 45.3174316),
        (6.5219256, 45.3176102),
        (6.5221216, 45.3177862),
        (6.5225114, 45.3180105),
        (6.5231628, 45.3184813),
        (6.5236938, 45.3187682),
        (6.5241808, 45.3191968),
        (6.5244890, 45.3197656),
        (6.5247164, 45.3201727),
        (6.5250230, 45.3206450),
        (6.5251761, 45.3208964),
        (6.5251324, 45.3212819),
        (6.5251929, 45.3218024),
        (6.5258114, 45.3225458),
        (6.5269514, 45.3230572),
        (6.5275646, 45.3231524),
        (6.5285838, 45.3238372),
        (6.5294963, 45.3244548),
        (6.5300600, 45.3246514),
        (6.5304261, 45.3250384),
        (6.5305019, 45.3254556),
        (6.5305143, 45.3258076),
        (6.5306505, 45.3259256),
    ]
}

#[fixture]
fn area00() -> Line {
    vec![
        (6.5222413, 45.3161829),
        (6.5216064, 45.3167159),
        (6.5215938, 45.3170868),
        (6.5219128, 45.3174177),
        (6.5227845, 45.3178285),
        (6.5231729, 45.3182172),
        (6.5237446, 45.3184370),
        (6.5242468, 45.3187213),
        (6.5246415, 45.3192543),
        (6.5248026, 45.3198294),
        (6.5251027, 45.3203979),
        (6.5253206, 45.3205023),
        (6.5259554, 45.3204113),
        (6.5256358, 45.3208932),
        (6.5251826, 45.3208788),
        (6.5247900, 45.3208465),
        (6.5241425, 45.3203891),
        (6.5241489, 45.3198095),
        (6.5237730, 45.3190366),
        (6.5230561, 45.3185481),
        (6.5220391, 45.3179795),
        (6.5213917, 45.3173022),
        (6.5211611, 45.3168802),
        (6.5210158, 45.3165427),
        (6.5214454, 45.3164849),
        (6.5222413, 45.3161829),
    ]
}

#[fixture]
fn area01() -> Line {
    vec![
        (6.5255638, 45.3210864),
        (6.5254957, 45.3213342),
        (6.5255037, 45.3216475),
        (6.5256822, 45.3220399),
        (6.5259734, 45.3223770),
        (6.5265170, 45.3226009),
        (6.5268323, 45.3227151),
        (6.5268049, 45.3228542),
        (6.5269514, 45.3230572),
        (6.5272505, 45.3231437),
        (6.5275646, 45.3231524),
        (6.5280724, 45.3231053),
        (6.5285501, 45.3234287),
        (6.5289256, 45.3236735),
        (6.5298220, 45.3242307),
        (6.5308093, 45.3246234),
        (6.5306764, 45.3250300),
        (6.5305831, 45.3255242),
        (6.5305349, 45.3257990),
        (6.5306505, 45.3259256),
        (6.5303398, 45.3259218),
        (6.5301249, 45.3252752),
        (6.5302497, 45.3250120),
        (6.5283468, 45.3239373),
        (6.5279437, 45.3240041),
        (6.5275207, 45.3236911),
        (6.5271330, 45.3233552),
        (6.5262581, 45.3229447),
        (6.5256983, 45.3226676),
        (6.5252061, 45.3221610),
        (6.5249810, 45.3216905),
        (6.5249375, 45.3213150),
        (6.5249574, 45.3209220),
        (6.5251761, 45.3208964),
        (6.5255638, 45.3210864),
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
            ref_: String::new(),
            name: "Piste 1".to_owned(),
            difficulty: Difficulty::Easy,
        },
        lines: vec![line0],
        areas: vec![area00, area01],
    }];
    let actual = PisteOut::list(&pistes);
    assert_eq!(actual, expected);
}
