use geo::Rect;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum ErrorType {
    InputError,
    OSMError,
    LogicError,
    ExternalError,
    NoSkiAreaAtLocation(Rect),
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct Error {
    #[serde(flatten)]
    type_: ErrorType,
    msg: String,
}

impl Error {
    pub fn new(type_: ErrorType, msg: String) -> Self {
        Error { type_, msg }
    }

    pub fn new_s(type_: ErrorType, msg: &str) -> Self {
        Error {
            type_,
            msg: msg.into(),
        }
    }

    pub fn convert<T>(type_: ErrorType, msg: &str, err: &T) -> Self
    where
        T: fmt::Display,
    {
        Error::new(type_, format!("{}: {}", msg, err))
    }

    pub fn get_type(&self) -> ErrorType {
        self.type_
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {}", self.type_, self.msg)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub fn convert_err<T, Err>(
    result: std::result::Result<T, Err>,
    error_type: ErrorType,
) -> Result<T>
where
    Err: std::error::Error,
{
    result.map_err(|err| Error::new(error_type, err.to_string()))
}
