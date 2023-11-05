use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{List, ListItem, ListState},
};

use super::BlockComponent;

pub trait MenuItem {
    fn to_menu_item(&self) -> Vec<Line<'_>>;
}

#[derive(Clone)]
pub struct Menu<T: MenuItem> {
    idx: usize,
    items: Vec<T>,
}

impl<T: MenuItem> Menu<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self { idx: 0, items }
    }

    fn next(&mut self) {
        self.idx = (self.idx + 1) % self.items.len();
    }

    fn previous(&mut self) {
        self.idx = match self.idx {
            0 => self.items.len() - 1,
            i => i - 1,
        };
    }

    pub fn selected(&self) -> &T {
        &self.items[self.idx]
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

impl<T: MenuItem> BlockComponent for Menu<T> {
    fn on_event(&mut self, key_event: crossterm::event::KeyEvent) -> super::HandleResult {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => self.next(),
            KeyCode::Char('k') | KeyCode::Up => self.previous(),
            _ => return Ok(super::HandleSuccess::Ignored),
        }

        Ok(super::HandleSuccess::Consumed)
    }

    fn render(
        &self,
        frame: &mut crate::terminal::Frame,
        area: ratatui::prelude::Rect,
        block: ratatui::widgets::Block,
    ) {
        let items = self
            .items
            .iter()
            .map(|i| ListItem::new(i.to_menu_item()))
            .collect::<Vec<_>>();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(
            list.block(block),
            area,
            &mut ListState::default().with_selected(Some(self.idx)),
        );
    }
}
