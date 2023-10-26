use crossterm::event::KeyEvent;
use ratatui::{prelude::Rect, widgets::Block};

use crate::terminal::Frame;

pub mod popup;
pub mod request_list;
pub mod response_panel;

pub enum HandleSuccess {
    Consumed,
    Ignored,
}

pub type HandleResult = anyhow::Result<HandleSuccess>;

pub trait Component {
    fn on_event(&mut self, key_event: KeyEvent) -> HandleResult;
    fn update(&mut self);
    fn render(&self, frame: &mut Frame, area: Rect);
}

pub trait BlockComponent {
    fn on_event(&mut self, key_event: KeyEvent) -> HandleResult;
    fn update(&mut self);
    fn render(&self, frame: &mut Frame, area: Rect, block: Block);
}