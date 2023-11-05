use ratatui::{
    prelude::{Constraint, Direction, Layout},
    widgets::Paragraph,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::ui::Legend;

use super::BlockComponent;

const INPUT_KEYMAPS: &[(&str, &str); 2] = &[("Enter", "Confirm"), ("Esc", "Cancel")];

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
        let [main_area, legend_area] = {
            let x = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(area);

            [x[0], x[1]]
        };
        let scroll = self.visual_scroll(main_area.width as usize);
        let legend = Legend::from(
            INPUT_KEYMAPS
                .iter()
                .map(|(a, b)| (a.to_owned().into(), b.to_owned().into()))
                .collect::<Vec<_>>(),
        );

        frame.render_widget(p.block(block), main_area);
        frame.render_widget(legend, legend_area);

        frame.set_cursor(
            // Put cursor past the end of the input text
            main_area.x + ((self.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            // Move one line down, from the border to the input line
            main_area.y + 1,
        )
    }
}
