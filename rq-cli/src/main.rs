use rq_core;

use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let unparsed_file = fs::read_to_string("./src/test.http")?;
    let http_file = rq_core::parser::parse(&unparsed_file);
    println!("file:\n{}", http_file.unwrap());

    Ok(())
}
