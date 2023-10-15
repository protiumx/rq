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

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap},
    Frame, Terminal,
};

use crate::app::{App, FocusState};

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
            if app.should_exit {
                return Ok(());
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn draw_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let frame_size = f.size();

    let direction = match frame_size.width >= frame_size.height {
        true => Direction::Horizontal,
        false => Direction::Vertical,
    };

    // Create two chunks with equal screen space
    let chunks = Layout::default()
        .direction(direction)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(frame_size);

    let (list_border_style, buffer_border_style) = match app.focus {
        FocusState::RequestsList => (Style::default().fg(Color::Blue), Style::default()),
        FocusState::ResponseBuffer => (Style::default(), Style::default().fg(Color::Blue)),
    };

    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(">> {} <<", app.file_path.as_str()))
        .border_style(list_border_style);

    let buffer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(buffer_border_style);

    let request_spans: Vec<ListItem> = app
        .list
        .items
        .iter()
        .map(|i| ListItem::new(draw_request(i)))
        .collect();

    let list = List::new(request_spans)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )
        .highlight_symbol("> ");

    let response_buffer = &mut app.buffers[app.list.selected_index()];
    let buffer_content = response_buffer.content();
    let buffer_y_scroll = response_buffer.scroll();

    let buffer = Paragraph::new(buffer_content)
        .wrap(Wrap { trim: true })
        .scroll((buffer_y_scroll, 0));

    f.render_stateful_widget(list.block(list_block), chunks[0], &mut app.list.state);
    f.render_widget(buffer.block(buffer_block), chunks[1]);
    f.render_stateful_widget(
        Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight),
        chunks[1],
        response_buffer.state(),
    )
}

fn draw_request(req: &'_ HttpRequest) -> Vec<Line<'_>> {
    let mut lines = vec![Line::from(vec![
        Span::styled(req.method.to_string(), Style::default().fg(Color::Green)),
        Span::raw(format!(" {} HTTP/{}", req.url, req.version)),
    ])];

    let headers: Vec<Line> = req
        .headers()
        .iter()
        .map(|(k, v)| Line::from(format!("{}: {}", k, v)))
        .collect();

    lines.extend(headers);
    // new line
    lines.push(Line::from(""));
    if !req.body.is_empty() {
        lines.push(Line::styled(
            req.body.as_str(),
            Style::default().fg(Color::Rgb(246, 69, 42)),
        ));
        lines.push(Line::from(""));
    }
    lines
}
