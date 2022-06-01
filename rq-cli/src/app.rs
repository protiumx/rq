use rq_core::parser::{HttpFile, HttpRequest};

use std::{
    error::Error,
    io,
    sync::Arc,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

struct App {
    list: ListState,
    requests: Vec<HttpRequest>,
    response_buffer: String,
}

impl App {
    fn new(http_file: HttpFile) -> Self {
        let mut list = ListState::default();
        list.select(Some(0));
        App {
            list,
            requests: http_file.requests,
            response_buffer: String::new(),
        }
    }

    fn selected_request(&self) -> HttpRequest {
        self.requests[self.list.selected().unwrap()].clone()
    }

    fn next(&mut self) {
        let mut i = self.list.selected().unwrap() + 1;
        if i >= self.requests.len() {
            i = 0;
        }

        self.list.select(Some(i));
    }

    fn previous(&mut self) {
        let mut i = self.list.selected().unwrap();
        if i == 0 {
            i = self.requests.len() - 1;
        } else {
            i -= 1;
        }
        self.list.select(Some(i));
    }
}

pub async fn run(http_file: HttpFile) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear().unwrap();

    let tick_rate = Duration::from_millis(250);
    let app = Arc::new(tokio::sync::Mutex::new(App::new(http_file)));
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
    app_a: Arc<tokio::sync::Mutex<App>>,
    tick_rate: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_tick = Instant::now();

    loop {
        let mut app = app_a.lock().await;
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                    KeyCode::Char('c') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            return Ok(());
                        }
                    }
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Enter => {
                        app.response_buffer = String::from("Loading...");
                        let req = app.selected_request();
                        let aca = app_a.clone();
                        tokio::spawn(async move {
                            let data = match rq_core::request::execute(&req).await {
                                Ok(r) => r,
                                Err(e) => e.to_string(),
                            };
                            let mut inner = aca.lock().await;
                            inner.response_buffer = data;
                        });
                    }
                    _ => {}
                },

                Event::Mouse(e) => {
                    app.response_buffer = format!("{:?}", e);
                }
                _ => {}
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let items: Vec<ListItem> = app
        .requests
        .iter()
        .map(|i| ListItem::new(draw_request(i)))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Requests"))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut app.list);

    f.render_widget(
        Paragraph::new(app.response_buffer.as_str())
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL)),
        chunks[1],
    );
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
