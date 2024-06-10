use ratatui::prelude::*;
use ratatui::widgets::*;
use std::any::Any;
use std::sync::mpsc;

use crate::logging::footstones::*;

pub use self::backend::handle_streaming_request;
pub use self::backend::ChatRequest;
pub use self::backend::ChatResponse;

mod backend;

pub struct Chat {
    pub name: String,
    pub messages: Vec<Message>,

    pub locked: bool,
    pub triggered: bool,

    pub channel: (mpsc::Sender<ChatResponse>, mpsc::Receiver<ChatResponse>),
}

impl Chat {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            messages: Vec::new(),
            locked: false,
            triggered: false,
            channel: mpsc::channel(),
        }
    }
}

pub struct Message {
    pub author: Author,
    pub content: String,
    metadata: Metadata,
}

impl Message {
    pub fn new(author: Author, content: &str) -> Self {
        Self {
            author,
            content: content.to_string(),
            metadata: Metadata(None),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Author {
    User,
    Bot,
}

struct Metadata(Option<Box<dyn Any>>);

impl Widget for &Chat {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let itemsspans = self.messages.iter().map(|msg| {
            let author = match msg.author {
                Author::User => Span::styled("User", Style::default().fg(Color::Yellow)),
                Author::Bot => Span::styled("Bot", Style::default().fg(Color::Green)),
            };
            Line::default().spans([author, Span::raw(": "), Span::raw(&msg.content)])
        });

        Widget::render(
            Paragraph::new(itemsspans.collect::<Vec<_>>())
                .block(Block::bordered().title(match self.locked {
                    true => format!("{} (Locked)", self.name).red(),
                    false => self.name.to_string().green(),
                }))
                .wrap(Wrap { trim: true }),
            area,
            buf,
        );
    }
}

impl Chat {
    pub fn reconsile(
        &mut self,
        request_handle: mpsc::Sender<(mpsc::Sender<ChatResponse>, ChatRequest)>,
    ) {
        if self.triggered {
            self.triggered = false;
            self.locked = true;
            let request = self.construct_request();

            info!("Sent request: {:?}", request);

            request_handle
                .send((self.channel.0.clone(), request))
                .unwrap();
        }

        if self.locked {
            match self.channel.1.try_recv() {
                Ok(value) => {
                    self.locked = !value.done;
                    if self.messages.last().unwrap().author == Author::User {
                        self.messages
                            .push(Message::new(Author::Bot, &value.message.content));
                    } else {
                        self.messages.last_mut().unwrap().content += &value.message.content;
                    }
                }
                Err(err) => match err {
                    mpsc::TryRecvError::Empty => {}
                    mpsc::TryRecvError::Disconnected => {
                        self.locked = false;
                    }
                },
            }
        }
    }

    fn construct_request(&self) -> ChatRequest {
        ChatRequest {
            model: self.name.clone(),
            messages: self
                .messages
                .iter()
                .map(|msg| backend::Message {
                    role: match msg.author {
                        Author::Bot => backend::Role::Assistant,
                        Author::User => backend::Role::User,
                    },
                    content: msg.content.clone(),
                })
                .collect(),
        }
    }
}
