use ratatui::{
    prelude::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Clear, Paragraph, Wrap},
};

use crate::ui::Legend;

use super::{BlockComponent, HandleResult, HandleSuccess};

const POPUP_KEYMAPS: &[(&str, &str); 1] = &[("Any", "Dismiss")];

#[derive(Clone)]
pub enum PopupContent {
    Info(String),
    Error(String),
}

#[derive(Clone)]
pub struct Popup {
    content: Option<PopupContent>,
    w_percent: u16,
    h_percent: u16,
}

impl Popup {
    pub fn set(&mut self, content: PopupContent) {
        self.content = Some(content);
    }

    pub fn new(content: Option<PopupContent>, w_percent: u16, h_percent: u16) -> Self {
        Self {
            content,
            w_percent,
            h_percent,
        }
    }
}

impl Default for Popup {
    fn default() -> Self {
        Self {
            content: None,
            w_percent: 40,
            h_percent: 25,
        }
    }
}

impl BlockComponent for Popup {
    fn on_event(&mut self, _key_event: crossterm::event::KeyEvent) -> HandleResult {
        match self.content {
            Some(_) => {
                self.content = None;
                Ok(HandleSuccess::Consumed)
            }
            None => Ok(HandleSuccess::Ignored),
        }
    }

    fn update(&mut self) {}

    fn render(
        &self,
        frame: &mut crate::terminal::Frame,
        area: ratatui::prelude::Rect,
        block: ratatui::widgets::Block,
    ) {
        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - self.w_percent) / 2),
                Constraint::Percentage(self.w_percent),
                Constraint::Percentage((100 - self.w_percent) / 2),
            ])
            .split(area)[1];
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - self.h_percent) / 2),
                Constraint::Percentage(self.h_percent),
                Constraint::Percentage((100 - self.h_percent) / 2),
            ])
            .split(popup_area)[1];

        let [popup_area, legend_area] = {
            let x = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(popup_area);

            [x[0], x[1]]
        };

        let (content, title, color) = match &self.content {
            None => {
                return;
            }
            Some(content) => match content {
                PopupContent::Info(content) => (content.as_str(), " info ", Color::Green),
                PopupContent::Error(content) => (content.as_str(), " error ", Color::Red),
            },
        };

        let p = Paragraph::new(content)
            .block(block.border_style(Style::default().fg(color)).title(title))
            .wrap(Wrap::default());

        let legend = Legend::from(
            POPUP_KEYMAPS
                .iter()
                .map(|(a, b)| (a.to_owned().into(), b.to_owned().into()))
                .collect::<Vec<_>>(),
        );

        frame.render_widget(Clear, popup_area);
        frame.render_widget(p, popup_area);
        frame.render_widget(legend, legend_area);
    }
}
