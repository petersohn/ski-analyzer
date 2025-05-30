use crate::error::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::AsRef;
use std::fs::OpenOptions;
use std::path::Path;

pub fn load_from_file<T: DeserializeOwned, P: AsRef<Path>>(
    path: P,
) -> Result<T> {
    let file = OpenOptions::new().read(true).open(path)?;
    Ok(serde_json::from_reader(file)?)
}

pub fn load_from_file_if_exists<T: DeserializeOwned, P: AsRef<Path>>(
    path: P,
) -> Result<Option<T>> {
    let file = OpenOptions::new().read(true).open(path);

    if let Err(err) = &file {
        if err.kind() == std::io::ErrorKind::NotFound {
            return Ok(None);
        }
    }
    let file2 = file?;
    Ok(Some(serde_json::from_reader(file2)?))
}

pub fn save_to_file<T: Serialize, P: AsRef<Path>>(
    value: &T,
    path: P,
) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    serde_json::to_writer(file, value)?;
    Ok(())
}

pub fn save_to_file_pretty<T: Serialize, P: AsRef<Path>>(
    value: &T,
    path: P,
) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}
