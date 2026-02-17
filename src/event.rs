use crossterm::event::{
  KeyCode::{BackTab, Backspace, Char, Delete, Down, End, Enter, Esc, Home, Insert, Left, Null, PageDown, PageUp, Right, Tab, Up, F},
  KeyEvent, KeyModifiers,
};
use futures::StreamExt;
use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
use serde::{Deserialize, Serialize};
use tokio::{
  sync::{mpsc, oneshot},
  task::JoinHandle,
};

#[derive(Debug, Clone, Copy)]
pub enum Event<I> {
  Input(I),
  Tick,
  Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub enum KeyCode {
  CtrlBackspace,
  CtrlDelete,
  AltBackspace,
  AltDelete,
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
  /// No-operation: used to disable a keybinding via `<Nop>` in config.
  /// This variant is never produced by actual keyboard input.
  Nop,
}

pub struct EventLoop {
  pub rx: mpsc::UnboundedReceiver<Event<KeyCode>>,
  pub tx: mpsc::UnboundedSender<Event<KeyCode>>,
  pub abort: mpsc::UnboundedSender<()>,
  pub tick_rate: std::time::Duration,
}

impl EventLoop {
  pub fn new(tick_rate: Option<std::time::Duration>, init: bool) -> Self {
    let (tx, rx) = mpsc::unbounded_channel();
    let _tx = tx.clone();
    let should_tick = tick_rate.is_some();
    let tick_rate = tick_rate.unwrap_or(std::time::Duration::from_millis(250));

    let (abort, mut abort_recv) = mpsc::unbounded_channel();

    if init {
      let mut reader = crossterm::event::EventStream::new();
      tokio::spawn(async move {
        loop {
          let delay = tokio::time::sleep(tick_rate);
          let event = reader.next();

          tokio::select! {
              _ = abort_recv.recv() => {
                  _tx.send(Event::Closed).unwrap_or_else(|_| warn!("Unable to send Closed event"));
                  _tx.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
                  break;
              },
              _ = delay, if should_tick => {
                  _tx.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
              },
              _ = _tx.closed() => break,
              maybe_event = event => {
                  if let Some(Ok(crossterm::event::Event::Key(key))) = maybe_event {
                      let key = match key.code {
                          Backspace => {
                              match key.modifiers {
                                  KeyModifiers::CONTROL => KeyCode::CtrlBackspace,
                                  KeyModifiers::ALT => KeyCode::AltBackspace,
                                  _ => KeyCode::Backspace,
                              }
                          },
                          Delete => {
                              match key.modifiers {
                                  KeyModifiers::CONTROL => KeyCode::CtrlDelete,
                                  KeyModifiers::ALT => KeyCode::AltDelete,
                                  _ => KeyCode::Delete,
                              }
                          },
                          Enter => KeyCode::Char('\n'),
                          Left => KeyCode::Left,
                          Right => KeyCode::Right,
                          Up => KeyCode::Up,
                          Down => KeyCode::Down,
                          Home => KeyCode::Home,
                          End => KeyCode::End,
                          PageUp => KeyCode::PageUp,
                          PageDown => KeyCode::PageDown,
                          Tab => KeyCode::Tab,
                          BackTab => KeyCode::BackTab,
                          Insert => KeyCode::Insert,
                          F(k) => KeyCode::F(k),
                          Null => KeyCode::Null,
                          Esc => KeyCode::Esc,
                          Char(c) => match key.modifiers {
                              KeyModifiers::NONE | KeyModifiers::SHIFT => KeyCode::Char(c),
                              KeyModifiers::CONTROL => KeyCode::Ctrl(c),
                              KeyModifiers::ALT => KeyCode::Alt(c),
                              _ => KeyCode::Null,
                          },
                          _ => KeyCode::Null,
                      };
                      _tx.send(Event::Input(key)).unwrap_or_else(|_| warn!("Unable to send {:?} event", key));
                  }
              }
          }
        }
      });
    }

    Self { tx, rx, tick_rate, abort }
  }
}
