use std::error;

use args::{handle_parsed_args, parse_args};

#[macro_use]
extern crate prettytable;

mod args;
mod errors;
mod store;

fn main() -> Result<(), Box<dyn error::Error + 'static>> {
    let parsed_args = parse_args()?;
    match handle_parsed_args(parsed_args) {
        Err(err) => {
            println!("Error: {}", err.description());
        }
        Ok(_) => {}
    }
    Ok(())
}
