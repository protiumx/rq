use ratatui::{
    prelude::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};
use rq_core::{
    parser::{HttpFile, HttpRequest},
    request::RequestResult,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use std::error::Error;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::ui::{ResponseComponent, StatefulList};

#[derive(Default)]
enum FocusState {
    #[default]
    RequestsList,
    ResponseBuffer,
}

pub struct App {
    res_rx: Receiver<(RequestResult, usize)>,
    req_tx: Sender<(HttpRequest, usize)>,

    responses: Vec<ResponseComponent>,
    list: StatefulList<HttpRequest>,
    should_exit: bool,
    file_path: String,
    focus: FocusState,
}

fn handle_requests(
    mut req_rx: Receiver<(HttpRequest, usize)>,
    res_tx: Sender<(RequestResult, usize)>,
) {
    tokio::spawn(async move {
        while let Some((req, i)) = req_rx.recv().await {
            let data = rq_core::request::execute(&req).await;
            res_tx.send((data, i)).await.unwrap();
        }
    });
}

impl App {
    pub fn new(file_path: String, http_file: HttpFile) -> Self {
        let (req_tx, req_rx) = channel::<(HttpRequest, usize)>(1);
        let (res_tx, res_rx) = channel::<(RequestResult, usize)>(1);

        handle_requests(req_rx, res_tx);

        let responses = std::iter::repeat_with(ResponseComponent::default)
            .take(http_file.requests.len())
            .collect();

        App {
            file_path,
            res_rx,
            req_tx,
            list: StatefulList::with_items(http_file.requests),
            responses,
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
                FocusState::ResponseBuffer => {
                    self.responses[self.list.selected_index()].scroll_down()
                }
            },
            KeyCode::Up | KeyCode::Char('k') => match self.focus {
                FocusState::RequestsList => self.list.previous(),
                FocusState::ResponseBuffer => {
                    self.responses[self.list.selected_index()].scroll_up()
                }
            },
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {
                self.focus = match self.focus {
                    FocusState::RequestsList => FocusState::ResponseBuffer,
                    FocusState::ResponseBuffer => FocusState::RequestsList,
                }
            }
            KeyCode::Enter => {
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

        let response = &self.responses[self.list.selected_index()];

        f.render_stateful_widget(list.block(list_block), chunks[0], &mut self.list.state());
        response.render(f, chunks[1], buffer_border_style);
    }

    pub fn update(&mut self) {
        // Poll for request responses
        if let Ok((res, i)) = self.res_rx.try_recv() {
            self.responses[i] = ResponseComponent::new(res);
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
