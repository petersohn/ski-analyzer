use super::saveable::Saver;
use serde::{de::DeserializeOwned, Serialize};
use ski_analyzer_lib::error::{convert_err, ErrorType, Result};
use std::fs::OpenOptions;
use std::io::BufReader;

pub struct FileSaver {
    filename: String,
}

impl FileSaver {
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

impl<T> Saver<T> for FileSaver
where
    T: Serialize + DeserializeOwned,
{
    fn save(&mut self, data: &T) -> Result<()> {
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
