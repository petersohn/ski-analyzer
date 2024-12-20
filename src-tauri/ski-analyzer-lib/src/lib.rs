pub mod config;
pub mod error;
pub mod gpx_analyzer;
pub mod osm_query;
pub mod osm_reader;
pub mod ski_area;
pub mod utils;

mod multipolygon;

#[cfg(test)]
mod multipolygon_test;
#[cfg(test)]
mod osm_reader_test;
