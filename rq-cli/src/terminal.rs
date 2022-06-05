use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rq_core::parser::HttpRequest;

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::app::App;

pub async fn start(app: App) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear().unwrap();

    let tick_rate = Duration::from_millis(250);
    let res = run_app(&mut terminal, app, tick_rate).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_tick = Instant::now();

    loop {
        app.tick();
        terminal.draw(|f| draw_ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            app.on_terminal_event(event::read()?).await?;
            if app.exited {
                return Ok(());
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn draw_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let request_spans: Vec<ListItem> = app
        .requests
        .iter()
        .map(|i| ListItem::new(draw_request(i)))
        .collect();

    let mut list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(">> {} <<", app.file_path.as_str()));
    let list = List::new(request_spans)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )
        .highlight_symbol("> ");

    let cursor_x = app.cursor_position.0;
    if chunks[0].x <= cursor_x && cursor_x < chunks[0].x + chunks[0].width {
        list_block = list_block.border_style(Style::default().fg(Color::Blue));
    }

    let mut buffer_block = Block::default().borders(Borders::ALL);
    if chunks[1].x <= cursor_x && cursor_x < chunks[1].x + chunks[1].width {
        buffer_block = buffer_block.border_style(Style::default().fg(Color::Blue));
    }
    let buffer = Paragraph::new(app.response_buffer.as_str()).wrap(Wrap { trim: true });

    f.render_stateful_widget(list.block(list_block), chunks[0], &mut app.list);
    f.render_widget(buffer.block(buffer_block), chunks[1]);
}

fn draw_request(req: &'_ HttpRequest) -> Vec<Spans<'_>> {
    let mut spans = vec![Spans::from(vec![
        Span::styled(req.method.to_string(), Style::default().fg(Color::Green)),
        Span::raw(format!(" {} HTTP/{}", req.url, req.version)),
    ])];

    let headers: Vec<Spans> = req
        .headers
        .iter()
        .map(|(k, v)| Spans::from(format!("{}: {}", k, v)))
        .collect();

    spans.extend(headers);
    // new line
    spans.push(Spans::from(""));
    if !req.body.is_empty() {
        spans.push(Spans::from(Span::styled(
            req.body.as_str(),
            Style::default().fg(Color::Rgb(246, 69, 42)),
        )));
        spans.push(Spans::from(""));
    }
    spans
}
