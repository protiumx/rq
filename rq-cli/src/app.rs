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

use crate::{
    components::{
        message_dialog::{Message, MessageDialog},
        popup::Popup,
        request_list::RequestList,
        response_panel::ResponsePanel,
        BlockComponent, HandleSuccess,
    },
    ui::Legend,
};

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
    message_popup: Popup<MessageDialog>,
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
            message_popup: Popup::new(MessageDialog::default()),
        }
    }

    async fn on_key_event(&mut self, event: KeyEvent) -> anyhow::Result<()> {
        match self.message_popup.on_event(event)? {
            HandleSuccess::Consumed => return Ok(()),
            HandleSuccess::Ignored => (),
        };

        // Consume event if App has the keymaps
        match event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_exit = true;
                return Ok(());
            }
            KeyCode::Char('c') => {
                if event.modifiers == KeyModifiers::CONTROL {
                    self.should_exit = true;
                }
                return Ok(());
            }
            KeyCode::Esc if matches!(self.focus, FocusState::ResponseBuffer) => {
                self.focus = FocusState::RequestsList;
                return Ok(());
            }
            KeyCode::Enter => {
                match self.focus {
                    FocusState::RequestsList => self.focus = FocusState::ResponseBuffer,
                    FocusState::ResponseBuffer => {
                        self.req_tx
                            .send((
                                self.request_list.selected().clone(),
                                self.request_list.selected_index(),
                            ))
                            .await?
                    }
                }
                return Ok(());
            }
            _ => (),
        };

        // Propagate event to siblings
        let event_result = match self.focus {
            FocusState::RequestsList => self.request_list.on_event(event),
            FocusState::ResponseBuffer => {
                self.responses[self.request_list.selected_index()].on_event(event)
            }
        };

        if let Err(e) = event_result {
            MessageDialog::push_message(Message::Error(e.to_string()));
        }

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
        let [list_chunk, response_chunk] = {
            let x = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_chunk);

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

        let legend = Legend::from(
            match self.focus {
                FocusState::RequestsList => REQUESTS_LIST_KEYMAPS.iter(),
                FocusState::ResponseBuffer => RESPONSE_BUFFER_KEYMAPS.iter(),
            }
            .map(|(a, b)| (a.to_owned().into(), b.to_owned().into()))
            .collect::<Vec<(String, String)>>(),
        );
        f.render_widget(legend, legend_chunk);

        self.message_popup
            .render(f, f.size(), Block::default().borders(Borders::ALL));
    }

    pub fn update(&mut self) {
        // Poll for request responses
        if let Ok((res, i)) = self.res_rx.try_recv() {
            self.responses[i] = ResponsePanel::from(res);
        }

        self.message_popup.update();
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
