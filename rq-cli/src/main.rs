use rq_core::parser::parse;

mod app;

use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("No files provided");
        return Ok(());
    }

    let file_content = fs::read_to_string(args[1].as_str())?;
    let http_file = parse(&file_content).unwrap();

    // clear screen
    //println!("\r\x1b[2J\r\x1b[H");

    //for req in http_file.requests {
    //println!("  {}\n", req.print());
    //}
    //println!("\x1b[H");
    //print!("\x1b[32m>");

    //io::stdout().flush().unwrap();

    app::run(http_file).await.unwrap();

    std::process::exit(0)
}
