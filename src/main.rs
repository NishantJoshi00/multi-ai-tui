use core::time::Duration;
use std::io;

use crossterm::event::{self, poll, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::Terminal;

pub mod app;
pub(crate) mod config;
mod logging;

macro_rules! tow {
    ($cloj:expr) => {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(3)
            .enable_all()
            .build()
            .unwrap()
            .block_on($cloj)
    };
}

use logging::footstones::*;

fn main() -> io::Result<()> {
    let _guard = logging::init();

    tracing::debug!("Starting Ratatui");

    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;
    terminal.show_cursor()?;

    let (tx, rx) = std::sync::mpsc::channel();

    let mut app = app::App::new(tx);

    let (ord_tx, ord_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let recv = ord_rx;

        tow!(async move {
            let mut pollable_futures = vec![];
            // while let Ok(sig) = recv.try_recv() {
            //     info!("Received signal: {:?}", sig);
            //     let fut = tokio::task::spawn(app::handle_streaming_request(sig));
            //     pollable_futures.push(fut);
            // }

            loop {
                let work = recv.try_iter();

                for sig in work {
                    info!("Received signal: {:?}", sig);
                    let fut = tokio::task::spawn(app::handle_streaming_request(sig));
                    pollable_futures.push(fut);
                }

                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
    });

    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            let mut state = app::State::default();
            frame.render_stateful_widget(&app, area, &mut state);
            frame.set_cursor(state.cursor.x, state.cursor.y);
        })?;

        if poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.modifiers, key.code) {
                    (event::KeyModifiers::CONTROL, event::KeyCode::Char('c')) => {
                        let _ = app.send.send(app::Signal::Exit);
                    }
                    _ => app.on_key(key),
                }
            }
        }
        app.reconsile(ord_tx.clone());

        if let Ok(signal) = rx.try_recv() {
            match signal {
                app::Signal::Exit => break,
            }
        }
    }

    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    app.errors.iter().enumerate().for_each(|(i, error)| {
        eprintln!("Error #{}", i);
        eprintln!("------------------------------------------------------------");
        eprintln!("{}", error);
        eprintln!("------------------------------------------------------------\n");
    });

    Ok(())
}
