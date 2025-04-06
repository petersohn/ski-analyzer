use std::fs::OpenOptions;
use std::io::BufReader;
use std::path::Path;

use gpx::Gpx;

use crate::error::Result;

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Gpx> {
    let file = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(file);
    Ok(gpx::read(reader)?)
}
