use anyhow::anyhow;
use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState, Wrap},
};
use rq_core::request::{Response, StatusCode};
use std::fmt::Write;
use tui_input::Input;

use super::{
    message_dialog::{Message, MessageDialog},
    popup::Popup,
    BlockComponent, HandleResult, HandleSuccess,
};

#[derive(Clone, Default)]
enum Content {
    Response(Response),
    #[default]
    Empty,
}

#[derive(Clone, Default)]
enum SaveMode {
    #[default]
    All,
    Body,
}

#[derive(Clone, Default)]
pub struct ResponsePanel {
    content: Content,
    scroll: u16,
    last_input: Option<String>,
    input_popup: Option<Popup<Input>>,
    save_mode: SaveMode,
}

impl From<Response> for ResponsePanel {
    fn from(value: Response) -> Self {
        Self {
            content: Content::Response(value),
            scroll: 0,
            last_input: None,
            input_popup: None,
            save_mode: SaveMode::All,
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

    fn get_last_input(&self) -> Option<&str> {
        self.last_input
            .as_ref()
            .and_then(|s| if s.is_empty() { None } else { Some(s.as_str()) })
    }

    fn save_to_file(&mut self) -> anyhow::Result<()> {
        let path = self.get_last_input().ok_or(anyhow!("Empty filename"))?;

        std::fs::write(path, self.to_string()?)?;

        MessageDialog::push_message(Message::Info(format!("Response saved to {}", path)));

        Ok(())
    }

    fn save_body_to_file(&mut self) -> anyhow::Result<()> {
        let path = self.get_last_input().ok_or(anyhow!("Empty filename"))?;

        std::fs::write(path, self.body()?)?;

        MessageDialog::push_message(Message::Info(format!("Response body saved to {}", path)));

        Ok(())
    }

    fn body(&self) -> anyhow::Result<String> {
        match &self.content {
            Content::Response(response) => Ok(response.body.clone()),
            Content::Empty => Err(anyhow!("Request not sent")),
        }
    }

    fn to_string(&self) -> anyhow::Result<String> {
        match &self.content {
            Content::Response(response) => {
                let headers = response
                    .headers
                    .iter()
                    .fold(String::new(), |mut acc, (k, v)| {
                        writeln!(acc, "{k}: {}", v.to_str().unwrap()).unwrap();
                        acc
                    });

                let s = format!(
                    "{} {}\n{headers}\n\n{}",
                    response.version, response.status, response.body
                );

                Ok(s)
            }
            Content::Empty => Err(anyhow!("Request not sent")),
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
                    self.last_input = Some(input_popup.value().into());
                    self.input_popup = None;

                    match self.save_mode {
                        SaveMode::All => self.save_to_file()?,
                        SaveMode::Body => self.save_body_to_file()?,
                    }

                    return Ok(HandleSuccess::Consumed);
                }
                KeyCode::Esc => {
                    self.last_input = None;
                    self.input_popup = None;

                    return Ok(HandleSuccess::Consumed);
                }
                _ => (),
            }
        }

        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.scroll_down(),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_up(),
            KeyCode::Char('s') => {
                self.save_mode = SaveMode::Body;
                self.input_popup = Some(Popup::new(Input::new("".into())));
            }
            KeyCode::Char('S') => {
                self.save_mode = SaveMode::All;
                self.input_popup = Some(Popup::new(Input::new("".into())));
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
        let content = match &self.content {
            Content::Response(response) => {
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
                for line in response.body.lines() {
                    lines.push(line.into());
                }

                lines
            }
            Content::Empty => vec![Line::styled("<Empty>", Style::default().fg(Color::Yellow))],
        };

        let content_length = content.len();

        let component = Paragraph::new(content)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0))
            .block(block);

        frame.render_widget(component, area);
        frame.render_stateful_widget(
            Scrollbar::default().orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight),
            area,
            &mut ScrollbarState::default()
                .position(self.scroll)
                .content_length(content_length as u16),
        );

        if let Some(input_popup) = self.input_popup.as_ref() {
            input_popup.render(
                frame,
                frame.size(),
                Block::default()
                    .borders(Borders::ALL)
                    .title(" output path "),
            );
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
