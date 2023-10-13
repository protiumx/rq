use rq_core::parser::parse;

mod app;
mod terminal;

use app::App;

use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        eprintln!("error: no files provided");
        std::process::exit(1);
    }

    let file_path = args[1].to_string();
    let file_content = fs::read_to_string(&file_path)?;

    let http_file = match parse(&file_content) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("parsing error: {e}");
            std::process::exit(1);
        }
    };

    let app = App::new(file_path, http_file);
    terminal::start(app).await?;

    std::process::exit(0)
}
