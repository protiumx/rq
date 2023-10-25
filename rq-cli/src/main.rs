use rq_core::parser::parse;

mod app;
mod components;
mod terminal;
mod ui;

use app::App;

use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = match env::args().nth(1) {
        Some(file_path) => file_path,
        None => {
            eprintln!("error: no files provided");
            std::process::exit(1);
        }
    };
    let file_content = fs::read_to_string(&file_path)?;

    let http_file = match parse(&file_content) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("parsing error: {e}");
            std::process::exit(1);
        }
    };

    let app = App::new(file_path, http_file);
    terminal::run(app).await?;

    std::process::exit(0)
}
