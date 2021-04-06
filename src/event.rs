use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use async_std::channel::unbounded;
use async_std::sync::Arc;
use async_std::task;
use futures::prelude::*;
use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,
    Tab,
}

#[derive(Debug, Clone, Copy)]
pub struct EventConfig {
    pub tick_rate: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    pub rx: async_std::channel::Receiver<Event<Key>>,
}

impl Events {
    pub fn with_config(config: EventConfig) -> Events {
        use crossterm::event::{KeyCode::*, KeyModifiers};
        let tick_rate = config.tick_rate;
        let (tx, rx) = unbounded::<Event<Key>>();
        task::spawn_local(async move {
            let mut reader = EventStream::new();

            loop {
                let mut delay = Delay::new(tick_rate).fuse();
                let mut event = reader.next().fuse();

                select! {
                    _ = delay => {
                        tx.send(Event::Tick).await.ok();
                    },
                    maybe_event = event => {
                        if let Some(Ok(event::Event::Key(key))) = maybe_event {
                            let key = match key.code {
                                Backspace => Key::Backspace,
                                Enter => Key::Char('\n'),
                                Left => Key::Left,
                                Right => Key::Right,
                                Up => Key::Up,
                                Down => Key::Down,
                                Home => Key::Home,
                                End => Key::End,
                                PageUp => Key::PageUp,
                                PageDown => Key::PageDown,
                                Tab => Key::Tab,
                                BackTab => Key::BackTab,
                                Delete => Key::Delete,
                                Insert => Key::Insert,
                                F(k) => Key::F(k),
                                Null => Key::Null,
                                Esc => Key::Esc,
                                Char(c) => match key.modifiers {
                                    KeyModifiers::NONE | KeyModifiers::SHIFT => Key::Char(c),
                                    KeyModifiers::CONTROL => Key::Ctrl(c),
                                    KeyModifiers::ALT => Key::Alt(c),
                                    _ => Key::Null,
                                },
                            };
                            tx.send(Event::Input(key)).await.unwrap();
                            task::sleep(Duration::from_millis(1)).await;
                            task::yield_now().await;
                        };
                    }
                }
            }
        });
        Events { rx }
    }

    /// Attempts to read an event.
    /// This function will block the current thread.
    pub async fn next(&self) -> Result<Event<Key>, async_std::channel::RecvError> {
        self.rx.recv().await
    }

    pub fn leave_tui_mode(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        terminal.show_cursor().unwrap();
    }

    pub fn enter_tui_mode(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();
        enable_raw_mode().unwrap();
        terminal.resize(terminal.size().unwrap()).unwrap();
    }
}
