use std::io::{stdout, Write};
use std::error::Error;

use curl::easy::Easy;

fn main() -> Result<(), Box<dyn Error>> {
    let mut easy = Easy::new();
    easy.url("https://www.rust-lang.org/")?;
    easy.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    })?;
    easy.perform()?;

    println!("{}", easy.response_code()?);

    Ok(())
}
