use rq_core::parser::parse;
use rq_core::request::HttpClient;

use std::env;
use std::fs;

use inquire::Select;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("No files provided");
        return Ok(());
    }

    let file_content = fs::read_to_string(args[1].as_str())?;
    let http_file = parse(&file_content).unwrap();

    let request = Select::new("Select requests to execute:", http_file.requests)
        .prompt()
        .unwrap();

    let client = HttpClient::new();
    client.execute(&request)?;

    Ok(())
}
