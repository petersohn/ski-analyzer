use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct InvalidInput {
    msg: String,
}

impl InvalidInput {
    pub fn new(msg: String) -> Self {
        InvalidInput { msg }
    }

    pub fn new_s(msg: &str) -> Self {
        InvalidInput { msg: msg.into() }
    }

    pub fn convert<T>(msg: &str, err: &T) -> Self
        where T: fmt::Display,
    {
        InvalidInput::new(format!("{}: {}", msg, err))
    }
}

impl fmt::Display for InvalidInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid OSM input: {}", self.msg)
    }
}

impl Error for InvalidInput {}

pub type Result<T> = std::result::Result<T, InvalidInput>;
