use ratatui::widgets::ListState;

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(0) => self.items.len() - 1,
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

impl<T> From<Vec<T>> for StatefulList<T> {
    fn from(value: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default().with_selected(Some(0)),
            items: value,
        }
    }
}
