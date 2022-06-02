use rq_core::parser::parse;

mod app;

use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        eprintln!("error: no files provided");
        std::process::exit(1);
    }

    let file_content = fs::read_to_string(args[1].as_str())?;
    let http_file = parse(&file_content)?;

    app::run(http_file).await?;

    std::process::exit(0)
}
