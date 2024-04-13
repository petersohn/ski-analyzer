use geo::{
    BoundingRect, CoordNum, LineString, MultiLineString, MultiPolygon, Point,
    Rect,
};
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use lift::parse_lift;
use piste::parse_pistes;

use crate::config::get_config;
use crate::error::{Error, ErrorType, Result};
use crate::osm_reader::{get_tag, Document};

mod lift;
mod piste;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointWithElevation {
    pub point: Point,
    pub elevation: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoundedGeometry<T, C = f64>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    pub item: T,
    pub bounding_rect: Rect<C>,
}

impl<T, C> BoundedGeometry<T, C>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    pub fn new(item: T) -> Result<Self> {
        let bounding_rect = item.bounding_rect().into().ok_or(Error::new_s(
            ErrorType::LogicError,
            "cannot calculate bounding rect",
        ))?;
        Ok(BoundedGeometry {
            item,
            bounding_rect,
        })
    }
}

impl<T, C> BoundingRect<C> for BoundedGeometry<T, C>
where
    C: CoordNum,
    T: BoundingRect<C>,
{
    type Output = Rect<C>;

    fn bounding_rect(&self) -> Self::Output {
        self.bounding_rect.bounding_rect()
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Lift {
    pub ref_: String,
    pub name: String,
    pub type_: String,
    pub line: BoundedGeometry<LineString>,
    pub begin_altitude: u32,
    pub end_altitude: u32,
    pub midstations: Vec<PointWithElevation>,
    pub can_go_reverse: bool,
    pub can_disembark: bool,
}

#[derive(
    Serialize,
    Deserialize,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    EnumString,
    strum_macros::Display,
)]
#[strum(serialize_all = "lowercase")]
pub enum Difficulty {
    #[strum(serialize = "")]
    Unknown,
    Novice,
    Easy,
    Intermediate,
    Advanced,
    Expert,
    Freeride,
}

#[derive(PartialEq, Eq, Hash, Clone, Serialize, Deserialize, Debug)]
pub struct PisteMetadata {
    pub ref_: String,
    pub name: String,
    pub difficulty: Difficulty,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PisteData {
    pub bounding_rect: Rect,
    pub areas: MultiPolygon,
    pub lines: MultiLineString,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Piste {
    #[serde(flatten)]
    pub metadata: PisteMetadata,
    #[serde(flatten)]
    pub data: PisteData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkiArea {
    pub name: String,
    pub lifts: Vec<Lift>,
    pub pistes: Vec<Piste>,
}

impl SkiArea {
    pub fn parse(doc: &Document) -> Result<Self> {
        let mut names: Vec<String> = Vec::new();
        let mut lifts = Vec::new();
        let config = get_config();

        for (_id, way) in &doc.elements.ways {
            if get_tag(&way.tags, "landuse") == "winter_sports" {
                names.push(get_tag(&way.tags, "name").to_string());
            }
        }

        if names.len() == 0 {
            return Err(Error::new_s(
                ErrorType::InputError,
                "ski area entity not found",
            ));
        } else if names.len() > 1 {
            return Err(Error::new(
                ErrorType::InputError,
                format!("ambiguous ski area: {:?}", names),
            ));
        }

        for (id, way) in &doc.elements.ways {
            match parse_lift(&doc, &id, &way) {
                Err(e) => eprintln!("Error parsing way {}: {}", id, e),
                Ok(None) => (),
                Ok(Some(lift)) => lifts.push(lift),
            }
        }

        if config.is_v() {
            eprintln!("Found {} lifts.", lifts.len());
        }

        let pistes = parse_pistes(&doc);

        if config.is_v() {
            eprintln!("Found {} pistes.", pistes.len());
        }

        Ok(SkiArea {
            name: names.remove(0),
            lifts,
            pistes,
        })
    }
}
