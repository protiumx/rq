use ratatui::widgets::{Clear, Paragraph};
use tui_input::{backend::crossterm::EventHandler, Input};

use super::BlockComponent;

impl BlockComponent for Input {
    fn on_event(&mut self, key_event: crossterm::event::KeyEvent) -> super::HandleResult {
        match self.handle_event(&crossterm::event::Event::Key(key_event)) {
            Some(_) => Ok(super::HandleSuccess::Consumed),
            None => Ok(super::HandleSuccess::Ignored),
        }
    }

    fn render(
        &self,
        frame: &mut crate::terminal::Frame,
        area: ratatui::prelude::Rect,
        block: ratatui::widgets::Block,
    ) {
        let p = Paragraph::new(self.value());
        let scroll = self.visual_scroll(area.width as usize);

        frame.render_widget(Clear, area);
        frame.render_widget(p.block(block), area);

        frame.set_cursor(
            // Put cursor past the end of the input text
            area.x + ((self.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            // Move one line down, from the border to the input line
            area.y + 1,
        )
    }
}
