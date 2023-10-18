use ratatui::{
    prelude::Rect,
    style::Style,
    widgets::{Block, Borders, ListState, Paragraph, Scrollbar, ScrollbarState, Wrap},
};
use rq_core::request::RequestResult;

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

    fn get_content(&self) -> String {
        match self.response.as_ref() {
            Some(response) => match response.as_ref() {
                Ok(response) => response.body.clone(),
                Err(e) => e.to_string(),
            },
            None => "Press Enter to send request".into(),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, border_style: Style) {
        let content = self.get_content();
        let content_length = content.lines().count();

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
