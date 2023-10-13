use rq_core::parser::{HttpFile, HttpRequest};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use std::error::Error;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use tui::widgets::ListState;

pub struct App {
    res_rx: Receiver<String>,
    req_tx: Sender<HttpRequest>,

    pub requests: Vec<HttpRequest>,
    pub response_buffer: String,
    pub list: ListState,
    pub cursor_position: (u16, u16),
    pub exited: bool,
    pub file_path: String,
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

        let mut list = ListState::default();
        list.select(Some(0));
        App {
            file_path,
            res_rx,
            req_tx,
            list,
            requests: http_file.requests,
            response_buffer: String::new(),
            cursor_position: (0, 0),
            exited: false,
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

    pub fn tick(&mut self) {
        if let Ok(res) = self.res_rx.try_recv() {
            self.response_buffer = res;
        }
    }

    pub async fn on_terminal_event(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
        match event {
            Event::Key(ev) => self.on_key_event(ev).await?,
            Event::Mouse(ev) => {
                self.on_mouse_event(ev);
            }
            _ => {}
        }
        Ok(())
    }

    fn on_mouse_event(&mut self, ev: MouseEvent) {
        if let MouseEventKind::Up(MouseButton::Left) = ev.kind {
            self.cursor_position = (ev.column, ev.row);
        }
    }

    async fn on_key_event(&mut self, event: KeyEvent) -> Result<(), Box<dyn Error>> {
        match event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.exited = true;
            }
            KeyCode::Char('c') => {
                if event.modifiers == KeyModifiers::CONTROL {
                    self.exited = true;
                }
            }
            KeyCode::Down => self.next(),
            KeyCode::Up => self.previous(),
            KeyCode::Enter => {
                self.response_buffer = String::from("Loading...");
                self.req_tx.send(self.selected_request()).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
