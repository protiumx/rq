use ratatui::widgets::{ListState, ScrollbarState};

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
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
}

#[derive(Clone, Default)]
pub struct ScrollBuffer {
    content: String,
    state: ScrollbarState,
    scroll: u16,
}

impl ScrollBuffer {
    pub fn next(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
        self.state = self.state.position(self.scroll)
    }

    pub fn prev(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
        self.state = self.state.position(self.scroll)
    }

    pub fn overwrite<T: AsRef<str>>(&mut self, new_content: T) {
        let new_content = new_content.as_ref();

        let line_count = new_content.lines().count();
        self.content = new_content.to_string();
        self.state = self.state.content_length(line_count as u16).position(0);
        self.scroll = 0;
    }

    pub fn content(&self) -> &str {
        self.content.as_str()
    }

    pub fn scroll(&self) -> u16 {
        self.scroll
    }

    pub fn state(&mut self) -> &mut ScrollbarState {
        &mut self.state
    }
}
