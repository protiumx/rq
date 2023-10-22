use anyhow::anyhow;
use ratatui::{
    prelude::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use rq_core::{
    parser::{HttpFile, HttpRequest},
    request::Response,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::ui::{Legend, ResponseComponent, StatefulList};

const REQUESTS_LIST_KEYMAPS: &[(&str, &str); 4] = &[
    ("q", "Quit"),
    ("j/↓", "Next request"),
    ("k/↑", "Prev request"),
    ("Enter", "Select request"),
];
const RESPONSE_BUFFER_KEYMAPS: &[(&str, &str); 7] = &[
    ("q", "Quit"),
    ("Esc", "Back to list"),
    ("j/↓", "Scroll down"),
    ("k/↑", "Scroll up"),
    ("Enter", "Send request"),
    ("s", "Save the body to file"),
    ("S", "Save entire request to file"),
];
const POPUP_KEYMAPS: &[(&str, &str); 1] = &[("Any", "Dismiss")];

#[derive(Default)]
enum FocusState {
    #[default]
    RequestsList,
    ResponseBuffer,
}

pub struct App {
    res_rx: Receiver<(anyhow::Result<Response>, usize)>,
    req_tx: Sender<(HttpRequest, usize)>,

    responses: Vec<ResponseComponent>,
    list: StatefulList<HttpRequest>,
    should_exit: bool,
    file_path: String,
    focus: FocusState,
    error: Option<String>,
}

fn handle_requests(
    mut req_rx: Receiver<(HttpRequest, usize)>,
    res_tx: Sender<(anyhow::Result<Response>, usize)>,
) {
    tokio::spawn(async move {
        while let Some((req, i)) = req_rx.recv().await {
            let data = rq_core::request::execute(&req)
                .await
                .map_err(|e| anyhow!(e));
            res_tx.send((data, i)).await.unwrap();
        }
    });
}

impl App {
    pub fn new(file_path: String, http_file: HttpFile) -> Self {
        let (req_tx, req_rx) = channel::<(HttpRequest, usize)>(1);
        let (res_tx, res_rx) = channel::<(anyhow::Result<Response>, usize)>(1);

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
            error: None,
        }
    }

    async fn on_key_event(&mut self, event: KeyEvent) -> anyhow::Result<()> {
        // Dismiss error
        if self.error.is_some() {
            self.error = None;
            return Ok(());
        }

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
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {}
            KeyCode::Enter => match self.focus {
                FocusState::RequestsList => self.focus = FocusState::ResponseBuffer,
                FocusState::ResponseBuffer => {
                    self.req_tx
                        .send((self.list.selected().clone(), self.list.selected_index()))
                        .await?
                }
            },
            KeyCode::Esc if matches!(self.focus, FocusState::ResponseBuffer) => {
                self.focus = FocusState::RequestsList
            }
            KeyCode::Char('s') => {
                if let Err(e) = self.save_body_to_file() {
                    self.error = Some(e.to_string());
                }
            }
            KeyCode::Char('S') => {
                if let Err(e) = self.save_to_file() {
                    self.error = Some(e.to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn save_to_file(&self) -> anyhow::Result<()> {
        let selected_response = &self.responses[self.list.selected_index()];

        std::fs::write("response.http", selected_response.to_string()?)?;

        Ok(())
    }

    fn save_body_to_file(&mut self) -> anyhow::Result<()> {
        let selected_response = &self.responses[self.list.selected_index()];

        std::fs::write("response.http", selected_response.body()?)?;

        Ok(())
    }

    pub fn draw(&self, f: &mut crate::terminal::Frame<'_>) {
        // Creates a bottom chunk for the legend
        let [main_chunk, legend_chunk] = {
            let x = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(f.size());

            [x[0], x[1]]
        };

        // Create two chunks with equal screen space
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_chunk);

        let (list_border_style, buffer_border_style) = match self.focus {
            _ if self.error.is_some() => (Style::default(), Style::default()),
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

        let legend = Legend::from(
            match self.focus {
                FocusState::RequestsList => REQUESTS_LIST_KEYMAPS.iter(),
                FocusState::ResponseBuffer => RESPONSE_BUFFER_KEYMAPS.iter(),
            }
            .map(|(a, b)| (a.to_owned().into(), b.to_owned().into()))
            .collect::<Vec<(String, String)>>(),
        );

        let response = &self.responses[self.list.selected_index()];

        f.render_stateful_widget(list.block(list_block), chunks[0], &mut self.list.state());
        if self.error.is_none() {
            f.render_widget(legend, legend_chunk);
        }
        response.render(f, chunks[1], buffer_border_style);

        if let Some(content) = self.error.as_ref() {
            let popup_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(f.size())[1];

            let [popup_chunk, legend_chunk] = {
                let x = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(37),
                        Constraint::Percentage(25),
                        Constraint::Length(1),
                        Constraint::Min(1),
                    ])
                    .split(popup_chunk);

                [x[1], x[2]]
            };

            let p = Paragraph::new(content.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red))
                        .title(" error "),
                )
                .wrap(Wrap::default());
            let legend = Legend::from(
                POPUP_KEYMAPS
                    .iter()
                    .map(|(a, b)| (a.to_owned().into(), b.to_owned().into()))
                    .collect::<Vec<_>>(),
            );

            f.render_widget(Clear, popup_chunk);
            f.render_widget(p, popup_chunk);
            f.render_widget(legend, legend_chunk);
        }
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

    pub async fn on_event(&mut self, e: crossterm::event::Event) -> anyhow::Result<()> {
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
