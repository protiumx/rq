use anyhow::anyhow;
use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState, Wrap},
};
use rq_core::request::{Content, Response, StatusCode};
use std::fmt::{Display, Write};
use tui_input::Input;

use super::{
    menu::{Menu, MenuItem},
    message_dialog::{Message, MessageDialog},
    popup::Popup,
    BlockComponent, HandleResult, HandleSuccess,
};

#[derive(Copy, Clone, Default)]
enum SaveOption {
    #[default]
    All,
    Body,
}

impl SaveOption {
    fn iterator() -> impl Iterator<Item = SaveOption> {
        [SaveOption::All, SaveOption::Body].iter().copied()
    }
}

impl Display for SaveOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveOption::All => write!(f, "Save entire response"),
            SaveOption::Body => write!(f, "Save response body"),
        }
    }
}

impl MenuItem for SaveOption {
    fn render(&self) -> Vec<Line<'_>> {
        vec![Line::from(self.to_string())]
    }
}

#[derive(Clone, Default)]
pub struct ResponsePanel {
    content: Option<Response>,
    scroll: u16,
    input_popup: Option<Popup<Input>>,
    save_option: SaveOption,
    save_menu: Option<Popup<Menu<SaveOption>>>,
}

impl From<Response> for ResponsePanel {
    fn from(value: Response) -> Self {
        let default = Self::default();

        Self {
            content: Some(value),
            ..default
        }
    }
}

impl ResponsePanel {
    fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    fn body(&self) -> anyhow::Result<Content> {
        match &self.content {
            Some(response) => Ok(response.body.clone()),
            None => Err(anyhow!("Request not sent")),
        }
    }

    fn to_string(&self) -> anyhow::Result<String> {
        match &self.content {
            Some(response) => {
                let headers = response
                    .headers
                    .iter()
                    .fold(String::new(), |mut acc, (k, v)| {
                        writeln!(acc, "{k}: {}", v.to_str().unwrap()).unwrap();
                        acc
                    });

                let s = format!(
                    "{} {}\n{headers}\n\n{}",
                    response.version,
                    response.status,
                    self.body()?
                );

                Ok(s)
            }
            None => Err(anyhow!("Request not sent")),
        }
    }
}

impl BlockComponent for ResponsePanel {
    fn on_event(&mut self, key_event: crossterm::event::KeyEvent) -> HandleResult {
        if let Some(input_popup) = self.input_popup.as_mut() {
            match input_popup.on_event(key_event)? {
                HandleSuccess::Consumed => return Ok(HandleSuccess::Consumed),
                HandleSuccess::Ignored => (),
            }

            match key_event.code {
                KeyCode::Enter => {
                    let file_path = input_popup.value().to_string();

                    let to_save = match self.save_option {
                        SaveOption::All => self.to_string()?.into(),
                        SaveOption::Body => match self.body()? {
                            Content::Bytes(b) => b,
                            Content::Text(t) => t.into(),
                        },
                    };

                    std::fs::write(&file_path, to_save)?;
                    self.input_popup = None;

                    MessageDialog::push_message(Message::Info(format!("Saved to {}", file_path)));

                    return Ok(HandleSuccess::Consumed);
                }
                KeyCode::Esc => {
                    self.input_popup = None;

                    return Ok(HandleSuccess::Consumed);
                }
                _ => (),
            }
        }

        if let Some(menu) = self.save_menu.as_mut() {
            match menu.on_event(key_event)? {
                HandleSuccess::Consumed => return Ok(HandleSuccess::Consumed),
                HandleSuccess::Ignored => (),
            }

            match key_event.code {
                KeyCode::Enter => {
                    self.save_option = *menu.selected();
                    self.save_menu = None;
                    self.input_popup = Some(Popup::new(Input::from("")));

                    return Ok(HandleSuccess::Consumed);
                }
                KeyCode::Esc => {
                    self.save_menu = None;

                    return Ok(HandleSuccess::Consumed);
                }
                _ => (),
            }
        }

        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.scroll_down(),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_up(),
            KeyCode::Char('s') => {
                self.save_menu = Some(Popup::new(Menu::new(SaveOption::iterator().collect())))
            }
            _ => return Ok(HandleSuccess::Ignored),
        };

        Ok(HandleSuccess::Consumed)
    }

    fn render(
        &self,
        frame: &mut crate::terminal::Frame,
        area: ratatui::prelude::Rect,
        block: ratatui::widgets::Block,
    ) {
        let body = match self.body() {
            Ok(x) => x.to_string(),
            Err(e) => e.to_string(),
        };

        let content = match &self.content {
            Some(response) => {
                let mut lines = vec![];

                // First line
                // <VERSION> <STATUS>
                lines.push(Line::from(vec![
                    response.version.clone().into(),
                    " ".into(),
                    Span::styled(
                        response.status.to_string(),
                        Style::default().fg(status_code_color(response.status)),
                    ),
                ]));

                // Headers
                // <KEY>: <VALUE>
                for (k, v) in &response.headers {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{k}"), Style::default().fg(Color::Blue)),
                        ": ".into(),
                        v.to_str().unwrap().into(),
                    ]))
                }

                // Body
                // with initial empty line
                lines.push(Line::from(""));
                for line in body.lines() {
                    lines.push(line.into());
                }

                lines
            }
            None => vec![Line::styled("<Empty>", Style::default().fg(Color::Yellow))],
        };

        let content_length = content.len();

        let component = Paragraph::new(content)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0))
            .block(block);

        frame.render_widget(component, area);

        if content_length as u16 > area.height {
            frame.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight),
                area,
                &mut ScrollbarState::default()
                    .position(self.scroll)
                    .content_length(content_length as u16),
            );
        }

        if let Some(input_popup) = self.input_popup.as_ref() {
            input_popup.render(
                frame,
                frame.size(),
                Block::default()
                    .borders(Borders::ALL)
                    .title(" output path "),
            );
        }

        if let Some(menu) = self.save_menu.as_ref() {
            menu.render(
                frame,
                frame.size(),
                Block::default().borders(Borders::ALL).title(" save menu "),
            )
        }
    }
}

fn status_code_color(status_code: StatusCode) -> Color {
    if status_code.is_success() {
        Color::Green
    } else if status_code.is_redirection() {
        Color::Yellow
    } else if status_code.is_client_error() || status_code.is_server_error() {
        Color::Red
    } else {
        Color::default()
    }
}
