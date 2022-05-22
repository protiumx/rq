use rq_core::parser::parse;

use std::env;
use std::fs;

use inquire::MultiSelect;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_content = fs::read_to_string(args[1].as_str())?;
    let http_file = parse(&file_content).unwrap();
    println!("file:\n{}", http_file);

    let ans = MultiSelect::new("Select requests to execute:", http_file.requests).prompt();
    println!("selected {:?}", ans);
    Ok(())
}
