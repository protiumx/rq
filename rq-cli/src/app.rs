use rq_core::parser::{HttpFile, HttpRequest};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use std::error::Error;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::ui::StatefulList;

#[derive(Default)]
pub enum FocusState {
    #[default]
    RequestsList,
    ResponseBuffer,
}

pub struct App {
    res_rx: Receiver<String>,
    req_tx: Sender<HttpRequest>,

    pub response_buffer: String,
    pub list: StatefulList<HttpRequest>,
    pub cursor_position: (u16, u16),
    pub should_exit: bool,
    pub file_path: String,
    pub focus: FocusState,
}

fn handle_requests(mut req_rx: Receiver<HttpRequest>, res_tx: Sender<String>) {
    tokio::spawn(async move {
        while let Some(req) = req_rx.recv().await {
            let data = match rq_core::request::execute(&req).await {
                Ok(r) => r,
                Err(e) => e.to_string(),
            };
            res_tx.send(data).await.unwrap();
        }
    });
}

impl App {
    pub fn new(file_path: String, http_file: HttpFile) -> Self {
        let (req_tx, req_rx) = channel::<HttpRequest>(1);
        let (res_tx, res_rx) = channel::<String>(1);

        handle_requests(req_rx, res_tx);

        App {
            file_path,
            res_rx,
            req_tx,
            list: StatefulList::with_items(http_file.requests),
            response_buffer: String::new(),
            cursor_position: (0, 0),
            should_exit: false,
            focus: FocusState::default(),
        }
    }

    pub fn tick(&mut self) {
        if let Ok(res) = self.res_rx.try_recv() {
            self.response_buffer = res;
        }
    }

    pub async fn on_terminal_event(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
        if let Event::Key(ev) = event {
            self.on_key_event(ev).await?;
        }
        Ok(())
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
            KeyCode::Down | KeyCode::Char('j')
                if matches!(self.focus, FocusState::RequestsList) =>
            {
                self.list.next()
            }
            KeyCode::Up | KeyCode::Char('k') if matches!(self.focus, FocusState::RequestsList) => {
                self.list.previous()
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {
                self.focus = match self.focus {
                    FocusState::RequestsList => FocusState::ResponseBuffer,
                    FocusState::ResponseBuffer => FocusState::RequestsList,
                }
            }
            KeyCode::Enter => {
                self.response_buffer = String::from("Loading...");
                self.req_tx.send(self.list.selected().clone()).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
