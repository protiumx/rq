use ratatui::{
    prelude::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders},
};
use rq_core::{
    parser::{HttpFile, HttpRequest},
    request::Response,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::components::{
    message_dialog::{Message, MessageDialog},
    popup::Popup,
    request_list::RequestList,
    response_panel::ResponsePanel,
    BlockComponent, HandleSuccess,
};

#[derive(Default)]
enum FocusState {
    #[default]
    RequestsList,
    ResponseBuffer,
}

pub struct App {
    res_rx: Receiver<(Response, usize)>,
    req_tx: Sender<(HttpRequest, usize)>,

    request_list: RequestList,
    responses: Vec<ResponsePanel>,
    should_exit: bool,
    file_path: String,
    focus: FocusState,
    message_popup: Option<Popup<MessageDialog>>,
}

fn handle_requests(mut req_rx: Receiver<(HttpRequest, usize)>, res_tx: Sender<(Response, usize)>) {
    tokio::spawn(async move {
        while let Some((req, i)) = req_rx.recv().await {
            let data = match rq_core::request::execute(&req).await {
                Ok(data) => data,
                Err(e) => {
                    MessageDialog::push_message(Message::Error(e.to_string()));
                    return;
                }
            };
            res_tx.send((data, i)).await.unwrap();
        }
    });
}

impl App {
    pub fn new(file_path: String, http_file: HttpFile) -> Self {
        let (req_tx, req_rx) = channel::<(HttpRequest, usize)>(1);
        let (res_tx, res_rx) = channel::<(Response, usize)>(1);

        handle_requests(req_rx, res_tx);

        let responses = std::iter::repeat(ResponsePanel::default())
            .take(http_file.requests.len())
            .collect();

        App {
            file_path,
            res_rx,
            req_tx,
            request_list: RequestList::from(http_file.requests),
            responses,
            should_exit: false,
            focus: FocusState::default(),
            message_popup: None,
        }
    }

    async fn on_key_event(&mut self, event: KeyEvent) -> anyhow::Result<()> {
        if let Some(popup) = self.message_popup.as_mut() {
            match popup.on_event(event)? {
                HandleSuccess::Consumed => {
                    self.message_popup = None;
                    return Ok(());
                }
                HandleSuccess::Ignored => (),
            };
        }

        // Propagate event to siblings
        let event_result = match self.focus {
            FocusState::RequestsList => self.request_list.on_event(event),
            FocusState::ResponseBuffer => {
                self.responses[self.request_list.selected_index()].on_event(event)
            }
        };

        match event_result {
            Ok(HandleSuccess::Consumed) => {
                return Ok(());
            }
            Ok(HandleSuccess::Ignored) => (),
            Err(e) => {
                MessageDialog::push_message(Message::Error(e.to_string()));
                return Ok(());
            }
        };

        match event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_exit = true;
            }
            KeyCode::Char('c') => {
                if event.modifiers == KeyModifiers::CONTROL {
                    self.should_exit = true;
                }
            }
            KeyCode::Esc if matches!(self.focus, FocusState::ResponseBuffer) => {
                self.focus = FocusState::RequestsList;
            }
            KeyCode::Enter => match self.focus {
                FocusState::RequestsList => self.focus = FocusState::ResponseBuffer,
                FocusState::ResponseBuffer => {
                    self.req_tx
                        .send((
                            self.request_list.selected().clone(),
                            self.request_list.selected_index(),
                        ))
                        .await?
                }
            },
            _ => (),
        };

        Ok(())
    }

    pub fn draw(&self, f: &mut crate::terminal::Frame<'_>) {
        // Create two chunks with equal screen space
        let [list_chunk, response_chunk] = {
            let x = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.size());

            [x[0], x[1]]
        };

        let (list_border_style, response_border_style) = match self.focus {
            FocusState::RequestsList => (Style::default().fg(Color::Blue), Style::default()),
            FocusState::ResponseBuffer => (Style::default(), Style::default().fg(Color::Blue)),
        };

        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(">> {} <<", self.file_path.as_str()))
            .border_style(list_border_style);
        self.request_list.render(f, list_chunk, list_block);

        let response_block = Block::default()
            .borders(Borders::ALL)
            .border_style(response_border_style);
        let response_panel = &self.responses[self.request_list.selected_index()];
        response_panel.render(f, response_chunk, response_block);

        if let Some(popup) = self.message_popup.as_ref() {
            popup.render(f, f.size(), Block::default().borders(Borders::ALL));
        }
    }

    pub fn update(&mut self) {
        // Poll for request responses
        if let Ok((res, i)) = self.res_rx.try_recv() {
            self.responses[i] = ResponsePanel::from(res);
        }

        if self.message_popup.is_none() {
            self.message_popup = MessageDialog::pop_message().map(Popup::new);
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
