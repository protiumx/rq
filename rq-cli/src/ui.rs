use ratatui::{
    prelude::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{ListState, Widget},
};

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

impl<T> From<Vec<T>> for StatefulList<T> {
    fn from(value: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default().with_selected(Some(0)),
            items: value,
        }
    }
}

pub struct Legend {
    keymaps: Vec<(String, String)>,
}

impl From<Vec<(String, String)>> for Legend {
    fn from(value: Vec<(String, String)>) -> Self {
        Legend { keymaps: value }
    }
}

impl Widget for Legend {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let spans = self
            .keymaps
            .iter()
            .flat_map(|(k, v)| {
                [
                    Span::styled(
                        format!(" {k} "),
                        Style::default().add_modifier(Modifier::REVERSED),
                    ),
                    format!(" {v} ").into(),
                ]
            })
            .collect::<Vec<_>>();

        let line = Line::from(spans);

        buf.set_line(area.x, area.y, &line, area.width);
    }
}