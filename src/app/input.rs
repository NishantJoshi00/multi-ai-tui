use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct Input {
    pub input: String,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Self {
            input: String::new(),
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Char(c)) => {
                self.input.push(c);
            }
            (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                self.input.push(c.to_ascii_uppercase());
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.input.pop();
            }
            _ => {}
        }
    }
}

impl StatefulWidget for &Input {
    type State = super::State;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let blocked_area = Block::bordered()
            .title("Input")
            .padding(Padding::uniform(1));

        let input_area = Paragraph::new(self.input.as_str()).block(blocked_area);

        state.cursor.x = area.x + self.input.len() as u16 + 2;
        state.cursor.y = area.y + 2;

        input_area.render(area, buf);
    }
}
