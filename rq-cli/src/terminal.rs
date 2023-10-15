use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, time::Duration};

use crate::app::App;

pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;

fn startup() -> std::io::Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> std::io::Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

async fn main_loop(app: &mut App) -> Result<(), Box<dyn Error>> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    loop {
        app.update();

        if event::poll(Duration::from_millis(250))? {
            app.on_event(event::read()?).await?;
        }

        t.draw(|f| {
            app.draw(f);
        })?;

        if app.should_exit() {
            break;
        }
    }

    Ok(())
}

pub async fn run(mut app: App) -> Result<(), Box<dyn Error>> {
    startup()?;
    let res = main_loop(&mut app).await;
    shutdown()?;

    res?;

    Ok(())
}
