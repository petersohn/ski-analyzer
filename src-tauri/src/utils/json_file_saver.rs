use super::saveable::Saver;
use serde::{de::DeserializeOwned, Serialize};
use ski_analyzer_lib::error::{convert_err, ErrorType, Result};
use std::fs::{create_dir_all, OpenOptions};
use std::io::BufReader;
use std::path::PathBuf;

pub struct FileSaver {
    filename: PathBuf,
}

impl FileSaver {
    pub fn new(filename: PathBuf) -> Self {
        Self { filename }
    }
}

impl<T> Saver<T> for FileSaver
where
    T: Serialize + DeserializeOwned,
{
    fn save(&mut self, data: &T) -> Result<()> {
        if let Some(parent) = self.filename.parent() {
            convert_err(create_dir_all(parent), ErrorType::ExternalError)?;
        }
        let file = convert_err(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&self.filename),
            ErrorType::ExternalError,
        )?;
        convert_err(serde_json::to_writer(file, data), ErrorType::ExternalError)
    }

    fn load(&self) -> Result<T> {
        let file = convert_err(
            OpenOptions::new().read(true).open(&self.filename),
            ErrorType::ExternalError,
        )?;
        let reader = BufReader::new(file);
        convert_err(serde_json::from_reader(reader), ErrorType::ExternalError)
    }
}
