use ratatui::prelude::*;
use ratatui::widgets::*;
use std::collections::HashMap;

pub struct Settings {
    pub settings_kv: HashMap<String, String>,
    pub hinting: Vec<String>,

    pub current_hint: Option<usize>,
}

impl Settings {
    pub fn new(settings_kv: HashMap<String, String>) -> Self {
        Self {
            settings_kv,
            hinting: Vec::new(),
            current_hint: None,
        }
    }
}

impl StatefulWidget for &Settings {
    type State = super::State;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let settings_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(20), Constraint::Min(1)].as_ref())
            .split(area);

        let settings_list = List::new(
            self.settings_kv
                .iter()
                .map(|(k, v)| {
                    let text = Line::default();
                    let text = text.spans([Span::bold(k.into()), Span::raw(":"), Span::raw(v)]);
                    ListItem::new(text)
                })
                .collect::<Vec<_>>(),
        )
        .block(Block::bordered().title("Settings"));

        let hint_list = List::new(
            self.hinting
                .iter()
                .map(|hint| ListItem::new(Span::raw(hint)))
                .collect::<Vec<_>>(),
        )
        .block(Block::bordered().title("Hints"));

        let settings_area = settings_layout[0];
        let hint_area = settings_layout[1];

        Widget::render(settings_list, settings_area, buf);
        Widget::render(hint_list, hint_area, buf);
    }
}
