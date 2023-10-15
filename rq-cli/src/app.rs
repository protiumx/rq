use ratatui::{
    prelude::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap},
};
use rq_core::parser::{HttpFile, HttpRequest};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use std::error::Error;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::ui::{ScrollBuffer, StatefulList};

#[derive(Default)]
enum FocusState {
    #[default]
    RequestsList,
    ResponseBuffer,
}

pub struct App {
    res_rx: Receiver<(String, usize)>,
    req_tx: Sender<(HttpRequest, usize)>,

    response_buffer: String,
    buffers: Vec<ScrollBuffer>,
    list: StatefulList<HttpRequest>,
    should_exit: bool,
    file_path: String,
    focus: FocusState,
}

fn handle_requests(mut req_rx: Receiver<(HttpRequest, usize)>, res_tx: Sender<(String, usize)>) {
    tokio::spawn(async move {
        while let Some((req, i)) = req_rx.recv().await {
            let data = match rq_core::request::execute(&req).await {
                Ok(r) => r,
                Err(e) => e.to_string(),
            };
            res_tx.send((data, i)).await.unwrap();
        }
    });
}

impl App {
    pub fn new(file_path: String, http_file: HttpFile) -> Self {
        let (req_tx, req_rx) = channel::<(HttpRequest, usize)>(1);
        let (res_tx, res_rx) = channel::<(String, usize)>(1);

        handle_requests(req_rx, res_tx);

        let buffers = std::iter::repeat(ScrollBuffer::default())
            .take(http_file.requests.len())
            .collect();

        App {
            file_path,
            res_rx,
            req_tx,
            buffers,
            list: StatefulList::with_items(http_file.requests),
            response_buffer: String::new(),
            should_exit: false,
            focus: FocusState::default(),
        }
    }

    async fn on_key_event(&mut self, event: KeyEvent) -> Result<(), Box<dyn Error>> {
        match event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_exit = true;
            }
            KeyCode::Char('c') => {
                if event.modifiers == KeyModifiers::CONTROL {
                    self.should_exit = true;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => match self.focus {
                FocusState::RequestsList => self.list.next(),
                FocusState::ResponseBuffer => self.buffers[self.list.selected_index()].next(),
            },
            KeyCode::Up | KeyCode::Char('k') => match self.focus {
                FocusState::RequestsList => self.list.previous(),
                FocusState::ResponseBuffer => self.buffers[self.list.selected_index()].prev(),
            },
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {
                self.focus = match self.focus {
                    FocusState::RequestsList => FocusState::ResponseBuffer,
                    FocusState::ResponseBuffer => FocusState::RequestsList,
                }
            }
            KeyCode::Enter => {
                self.response_buffer = String::from("Loading...");
                self.req_tx
                    .send((self.list.selected().clone(), self.list.selected_index()))
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn draw(&self, f: &mut crate::terminal::Frame<'_>) {
        // Create two chunks with equal screen space
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        let (list_border_style, buffer_border_style) = match self.focus {
            FocusState::RequestsList => (Style::default().fg(Color::Blue), Style::default()),
            FocusState::ResponseBuffer => (Style::default(), Style::default().fg(Color::Blue)),
        };

        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(">> {} <<", self.file_path.as_str()))
            .border_style(list_border_style);

        let buffer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(buffer_border_style);

        let request_spans: Vec<ListItem> = self
            .list
            .items()
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

        let response_buffer = &self.buffers[self.list.selected_index()];
        let buffer_content = response_buffer.content();
        let buffer_y_scroll = response_buffer.scroll();

        let buffer = Paragraph::new(buffer_content)
            .wrap(Wrap { trim: true })
            .scroll((buffer_y_scroll, 0));

        f.render_stateful_widget(list.block(list_block), chunks[0], &mut self.list.state());
        f.render_widget(buffer.block(buffer_block), chunks[1]);
        f.render_stateful_widget(
            Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight),
            chunks[1],
            &mut response_buffer.state(),
        )
    }

    pub fn update(&mut self) {
        // Poll for request responses
        if let Ok((res, i)) = self.res_rx.try_recv() {
            self.buffers[i].overwrite(res);
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub async fn on_event(&mut self, e: crossterm::event::Event) -> Result<(), Box<dyn Error>> {
        if let Event::Key(e) = e {
            self.on_key_event(e).await?;
        }
        Ok(())
    }
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
