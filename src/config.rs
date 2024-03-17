use crate::error::{InvalidInput, Result};

use clap::Args;

use std::sync::OnceLock;

#[derive(Clone, Args)]
pub struct Config {
    /// Print more information
    #[arg(short, long)]
    pub verbose: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    let ret: &Config = CONFIG.get().unwrap();
    ret
}

pub fn set_config(config: Config) -> Result<()> {
    CONFIG
        .set(config)
        .or(Err(InvalidInput::new_s("config already set")))
}
