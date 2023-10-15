use rq_core::parser::{HttpFile, HttpRequest};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use std::error::Error;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::ui::{ScrollBuffer, StatefulList};

#[derive(Default)]
pub enum FocusState {
    #[default]
    RequestsList,
    ResponseBuffer,
}

pub struct App {
    res_rx: Receiver<(String, usize)>,
    req_tx: Sender<(HttpRequest, usize)>,

    pub response_buffer: String,
    pub buffers: Vec<ScrollBuffer>,
    pub list: StatefulList<HttpRequest>,
    pub cursor_position: (u16, u16),
    pub should_exit: bool,
    pub file_path: String,
    pub focus: FocusState,
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
            cursor_position: (0, 0),
            should_exit: false,
            focus: FocusState::default(),
        }
    }

    pub fn tick(&mut self) {
        if let Ok((res, i)) = self.res_rx.try_recv() {
            self.buffers[i].overwrite(res);
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
}
