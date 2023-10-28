use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
};
use rq_core::parser::HttpRequest;

use crate::{terminal::Frame, ui::StatefulList};

use super::{BlockComponent, HandleResult, HandleSuccess};

pub struct RequestList {
    list: StatefulList<HttpRequest>,
}

impl From<Vec<HttpRequest>> for RequestList {
    fn from(value: Vec<HttpRequest>) -> Self {
        Self {
            list: StatefulList::from(value),
        }
    }
}

impl BlockComponent for RequestList {
    fn on_event(&mut self, key_event: KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.list.next(),
            KeyCode::Up | KeyCode::Char('k') => self.list.previous(),
            _ => return Ok(HandleSuccess::Ignored),
        };

        Ok(HandleSuccess::Consumed)
    }

    fn render(&self, frame: &mut Frame, area: Rect, block: Block) {
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

        frame.render_stateful_widget(list.block(block), area, &mut self.list.state());
    }
}

impl RequestList {
    pub fn selected_index(&self) -> usize {
        self.list.selected_index()
    }

    pub fn selected(&self) -> &HttpRequest {
        self.list.selected()
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
