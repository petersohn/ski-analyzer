use crate::error::{Error, ErrorType, Result};

use clap::Args;

use std::sync::OnceLock;

#[derive(Clone, Args)]
pub struct Config {
    /// Print more information.
    /// -v: status report.
    /// -vv: anomalies.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

impl Config {
    pub fn is_v(&self) -> bool {
        self.verbose >= 1
    }

    pub fn is_vv(&self) -> bool {
        self.verbose >= 2
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    let ret: &Config = CONFIG.get().unwrap();
    ret
}

pub fn set_config(config: Config) -> Result<()> {
    CONFIG.set(config).or(Err(Error::new_s(
        ErrorType::LogicError,
        "config already set",
    )))
}
