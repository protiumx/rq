use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use ratatui::{
    prelude::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Paragraph, Wrap},
};

use crate::ui::Legend;

use super::{BlockComponent, HandleResult, HandleSuccess};

lazy_static! {
    static ref MESSAGES: Arc<Mutex<VecDeque<Message>>> = Arc::new(Mutex::new(VecDeque::new()));
}

const POPUP_KEYMAPS: &[(&str, &str); 1] = &[("Any", "Dismiss")];

#[derive(Clone)]
pub enum Message {
    Info(String),
    Error(String),
}

#[derive(Clone)]
pub struct MessageDialog {
    content: Message,
}

impl MessageDialog {
    pub fn push_message(content: Message) {
        MESSAGES.as_ref().lock().unwrap().push_back(content);
    }

    pub fn pop_message() -> Option<Self> {
        MESSAGES
            .as_ref()
            .lock()
            .unwrap()
            .pop_front()
            .map(|content| Self { content })
    }
}

impl BlockComponent for MessageDialog {
    fn on_event(&mut self, _key_event: crossterm::event::KeyEvent) -> HandleResult {
        Ok(HandleSuccess::Consumed)
    }

    fn render(
        &self,
        frame: &mut crate::terminal::Frame,
        area: ratatui::prelude::Rect,
        block: ratatui::widgets::Block,
    ) {
        let [main_area, legend_area] = {
            let x = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(area);

            [x[0], x[1]]
        };

        let (content, title, color) = match &self.content {
            Message::Info(content) => (content.as_str(), " info ", Color::Green),
            Message::Error(content) => (content.as_str(), " error ", Color::Red),
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

        frame.render_widget(p, main_area);
        frame.render_widget(legend, legend_area);
    }
}
