use core::mem::swap;
use core::str::FromStr;
use std::sync::mpsc;

use nom::bytes::complete::{escaped, tag};
use nom::character::complete::{alphanumeric1, one_of};
use nom::combinator::{cut, opt};
use nom::error::{convert_error, ContextError, ParseError, VerboseError};
use nom::multi::separated_list0;
use nom::{branch, sequence, IResult, Parser};

use super::chat::{Chat, ChatRequest, ChatResponse, Message};
use super::{App, Signal};

impl App {
    pub fn reconsile(
        &mut self,
        request_handler: mpsc::Sender<(mpsc::Sender<ChatResponse>, ChatRequest)>,
    ) {
        let current = self.buffer.pop_front();
        if let Some(current) = current {
            let entry = root_parser::<VerboseError<&str>>(&current);

            match entry {
                Ok((_, entry)) => {
                    entry.reconsile(self);
                }
                Err(nom::Err::Error(e) | nom::Err::Failure(e)) => {
                    self.errors.push(convert_error(current.as_ref(), e));
                }
                _ => {}
            }
        }

        self.chats.iter_mut().for_each(|chat| {
            chat.reconsile(request_handler.clone());
        });
    }
}

#[derive(Debug)]
pub enum Entry {
    Command { command: Command, args: Vec<String> },
    Message { message: String },
}

impl Entry {
    fn reconsile(self, app: &mut App) {
        match self {
            Entry::Command { command, args } => match command {
                Command::Exit => {
                    app.send.send(Signal::Exit).unwrap();
                }
                Command::CreateChat => {
                    let name = args.first();
                    if let Some(name) = name {
                        app.chats.push(Chat::new(name))
                    } else {
                        app.errors.push("Chat name is required".to_string());
                    }
                }
                Command::DeleteChat => {
                    let name = args.first();
                    if let Some(name) = name {
                        app.chats.retain(|chat| chat.name != *name || chat.locked);
                    } else {
                        app.errors.push("Chat name is required".to_string());
                    }
                }
                Command::Clear => {
                    app.chats.iter_mut().for_each(|chat| {
                        chat.messages.clear();
                        chat.triggered = false;
                        chat.locked = false;
                        let mut my_channel = mpsc::channel();

                        swap(&mut chat.channel, &mut my_channel);
                        drop(my_channel);
                    });
                }
            },
            Entry::Message { message } => {
                app.chats.iter_mut().for_each(|chat| {
                    if !chat.locked {
                        chat.messages
                            .push(Message::new(super::chat::Author::User, &message));
                        chat.triggered = true;
                    }
                });
            }
        }
    }
}

pub fn root_parser<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Entry, E> {
    branch::alt((
        command_parser.map(|(command, args)| Entry::Command {
            command,
            args: args.iter().map(|s| s.to_string()).collect(),
        }),
        message_parser.map(|message| Entry::Message {
            message: message.to_string(),
        }),
    ))
    .parse(input)
}

#[derive(Debug)]
pub enum Command {
    Exit,
    CreateChat,
    DeleteChat,
    Clear,
}

impl Command {
    fn parse<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, Command, E> {
        branch::alt((
            tag("exit").map(|_| Command::Exit),
            tag("create").map(|_| Command::CreateChat),
            tag("delete").map(|_| Command::DeleteChat),
            tag("brainwash").map(|_| Command::Clear),
        ))
        .parse(input)
    }
}

pub fn message_parser<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    nom::character::complete::not_line_ending.parse(input)
}

fn command_parser<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, (Command, Vec<&'a str>), E> {
    sequence::preceded(
        tag("/"),
        cut(sequence::pair(
            Command::parse,
            opt(sequence::preceded(
                tag(" "),
                separated_list0(tag(" "), string_parser),
            ))
            .map(|value| value.unwrap_or_default()),
        )),
    )
    .parse(input)
}

fn string_parser<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    escaped(alphanumeric1, '\\', one_of("\"\\ ")).parse(input)
}
