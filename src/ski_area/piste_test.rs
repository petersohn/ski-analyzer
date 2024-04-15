use super::piste::parse_pistes;
use super::Piste;
use crate::osm_reader::{Document, Node, Tags, Way};

type Point = (f64, f64);
type Line = Vec<Point>;

struct WayDef {
    line: Line,
    tags: Tags,
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

    fn add_way(&mut self, line: Line, tags: Tags) {
        let mut nodes: Vec<u64> = Vec::new();
        nodes.reserve(line.len());
        for p in line {
            nodes.push(self.add_node(p));
        }

        let id = self.get_id();
        self.document.elements.ways.insert(id, Way { nodes, tags });
    }
}

fn create_document(ways: Vec<WayDef>) -> Document {
    let mut builder = DocumentBuilder::new();
    for way in ways {
        builder.add_way(way.line, way.tags);
    }
    builder.document
}
