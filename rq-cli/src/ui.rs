use std::fmt::Display;

use ratatui::{
    prelude::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListState, Paragraph, Scrollbar, ScrollbarState, Wrap},
};
use rq_core::request::{RequestResult, StatusCode};

use crate::terminal::Frame;

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default().with_selected(Some(0)),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) if i == 0 => self.items.len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected(&self) -> &T {
        let i = self.state.selected().unwrap_or(0);
        &self.items[i]
    }

    pub fn selected_index(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    pub fn state(&self) -> ListState {
        self.state.clone()
    }

    pub fn items(&self) -> &[T] {
        self.items.as_slice()
    }
}

#[derive(Default)]
pub struct ResponseComponent {
    response: Option<RequestResult>,
    scroll: u16,
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

impl ResponseComponent {
    pub fn new(response: RequestResult) -> Self {
        ResponseComponent {
            response: Some(response),
            scroll: 0,
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    fn get_content(&self) -> Vec<Line<'_>> {
        match self.response.as_ref() {
            Some(response) => match response.as_ref() {
                Ok(response) => {
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
                Err(e) => vec![Line::styled(e.to_string(), Style::default().fg(Color::Red))],
            },
            None => vec![Line::styled(
                "Press Enter to send request",
                Style::default().fg(Color::Yellow),
            )],
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, border_style: Style) {
        let content = self.get_content();
        let content_length = content.len();

        let component = Paragraph::new(self.get_content())
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

        f.render_widget(component, area);
        f.render_stateful_widget(
            Scrollbar::default().orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight),
            area,
            &mut ScrollbarState::default()
                .position(self.scroll)
                .content_length(content_length as u16),
        )
    }
}

impl Display for ResponseComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.response.as_ref() {
            Some(response) => match response.as_ref() {
                Ok(response) => {
                    let headers = response
                        .headers
                        .iter()
                        .map(|(k, v)| format!("{k}: {}\n", v.to_str().unwrap()))
                        .collect::<String>();

                    write!(
                        f,
                        "{} {}\n{headers}\n\n{}",
                        response.version, response.status, response.body
                    )
                }
                Err(e) => write!(f, "error: {e}"),
            },
            None => write!(f, "Request not sent"),
        }
    }
}
