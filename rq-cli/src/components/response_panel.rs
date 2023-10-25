use anyhow::anyhow;
use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState, Wrap},
};
use rq_core::request::{Response, StatusCode};

use super::{
    popup::{Popup, PopupContent},
    BlockComponent, HandleResult, HandleSuccess,
};

#[derive(Clone, Default)]
enum Content {
    Response(Response),
    Error(String),
    #[default]
    Empty,
}

#[derive(Clone, Default)]
pub struct ResponsePanel {
    content: Content,
    scroll: u16,
    popup: Popup,
}

impl From<anyhow::Result<Response>> for ResponsePanel {
    fn from(value: anyhow::Result<Response>) -> Self {
        let content = match value {
            Ok(response) => Content::Response(response),
            Err(e) => Content::Error(e.to_string()),
        };

        Self {
            content,
            scroll: 0,
            popup: Popup::new(None, 75, 25),
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

    fn save_to_file(&mut self) -> anyhow::Result<()> {
        let path = "response.http";
        std::fs::write(path, self.to_string()?)?;

        self.popup
            .set(PopupContent::Info(format!("Response saved to {}", path)));

        Ok(())
    }

    fn save_body_to_file(&mut self) -> anyhow::Result<()> {
        let path = "response.http";
        std::fs::write(path, self.body()?)?;

        self.popup
            .set(PopupContent::Info(format!("Response saved to {}", path)));

        Ok(())
    }

    fn body(&self) -> anyhow::Result<String> {
        match &self.content {
            Content::Response(response) => Ok(response.body.clone()),
            Content::Error(e) => Err(anyhow!(e.clone())),
            Content::Empty => Err(anyhow!("Request not sent")),
        }
    }

    fn to_string(&self) -> anyhow::Result<String> {
        match &self.content {
            Content::Response(response) => {
                let headers = response
                    .headers
                    .iter()
                    .map(|(k, v)| format!("{k}: {}\n", v.to_str().unwrap()))
                    .collect::<String>();

                let s = format!(
                    "{} {}\n{headers}\n\n{}",
                    response.version, response.status, response.body
                );

                Ok(s)
            }
            Content::Error(e) => Err(anyhow!(e.clone())),
            Content::Empty => Err(anyhow!("Request not sent")),
        }
    }
}

impl BlockComponent for ResponsePanel {
    fn on_event(&mut self, key_event: crossterm::event::KeyEvent) -> HandleResult {
        match self.popup.on_event(key_event)? {
            HandleSuccess::Consumed => return Ok(HandleSuccess::Consumed),
            HandleSuccess::Ignored => (),
        }

        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.scroll_down(),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_up(),
            KeyCode::Char('s') => self.save_body_to_file()?,
            KeyCode::Char('S') => self.save_to_file()?,
            _ => return Ok(HandleSuccess::Ignored),
        };

        Ok(HandleSuccess::Consumed)
    }

    fn update(&mut self) {}

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
            Content::Error(e) => vec![Line::styled(e.to_string(), Style::default().fg(Color::Red))],
            Content::Empty => vec![Line::styled("<Empty>", Style::default().fg(Color::Yellow))],
        };

        let content_length = content.len();
        let title = match &self.content {
            Content::Error(_) => " error ",
            _ => "",
        };

        let component = Paragraph::new(content)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0))
            .block(block.title(title));

        frame.render_widget(component, area);
        frame.render_stateful_widget(
            Scrollbar::default().orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight),
            area,
            &mut ScrollbarState::default()
                .position(self.scroll)
                .content_length(content_length as u16),
        );

        self.popup
            .render(frame, area, Block::default().borders(Borders::ALL));
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
