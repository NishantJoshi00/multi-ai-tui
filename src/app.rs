mod chat;
mod input;
mod reconsile;
mod settings;

use crate::config::Config;
use chat::Chat;
use crossterm::event::{self, KeyEvent};
use input::Input;
use ratatui::widgets::Widget;
use settings::Settings;
use std::collections::VecDeque;
use std::sync::mpsc;

pub use chat::handle_streaming_request;
use ratatui::prelude::*;

pub struct App {
    pub settings: Settings,
    pub chats: Vec<Chat>,
    pub input: Input,
    pub buffer: VecDeque<String>,
    pub view_ctx: ViewCtx,
    pub config: Config,
    pub send: mpsc::Sender<Signal>,

    pub errors: Vec<String>,
}

pub enum Signal {
    Exit,
}

pub enum ViewCtx {
    Input,
    Complete,
}

impl App {
    pub fn new(coms: mpsc::Sender<Signal>) -> Self {
        let config = Config::default();
        let settings = Settings::new(config.clone().into());
        let input = Input::new();

        Self {
            settings,
            chats: Vec::new(),
            input,
            view_ctx: ViewCtx::Input,
            config,
            buffer: VecDeque::new(),
            send: coms,

            errors: Vec::new(),
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        match self.view_ctx {
            ViewCtx::Input => match key.code {
                event::KeyCode::Enter => {
                    let text = self.input.input.clone();
                    self.input.input.clear();
                    self.buffer.push_back(text);
                }
                _ => {
                    self.input.on_key(key);
                }
            },
            ViewCtx::Complete => {}
        }
    }
}

pub struct State {
    pub cursor: CursorLoc,
}

pub struct CursorLoc {
    pub x: u16,
    pub y: u16,
}

impl Default for State {
    fn default() -> Self {
        Self {
            cursor: CursorLoc { x: 0, y: 0 },
        }
    }
}

impl StatefulWidget for &App {
    type State = State;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // The display will be chunked into 3 parts:
        // 1. The chat area which would be around w x h (75% x 80%) or the top right
        // 2. The input area which would be around w x h (100% x 20%) or the bottom
        // 3. The hint area which would be around w x h (25% x 80%) or the top left

        // let's first chunk vertically
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(vertical_chunks[0]);

        let input_area = vertical_chunks[1];

        let settings_area = horizontal_chunks[0];

        let chat_area = horizontal_chunks[1];

        match self.view_ctx {
            ViewCtx::Input => {
                self.settings
                    .render(settings_area, buf, &mut State::default());
                self.input.render(input_area, buf, state);
            }
            ViewCtx::Complete => {
                self.settings.render(settings_area, buf, state);
                self.input.render(input_area, buf, &mut State::default());
            }
        }

        let constaints = self
            .chats
            .iter()
            .map(|_| Constraint::Percentage(100 / self.chats.len() as u16))
            .collect::<Vec<_>>();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constaints)
            .split(chat_area);

        self.chats
            .iter()
            .zip(chunks.iter())
            .for_each(|(chat, area)| {
                chat.render(*area, buf);
            });
    }
}
