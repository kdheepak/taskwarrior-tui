use crossterm::event::{
  KeyCode::{BackTab, Backspace, Char, Delete, Down, End, Enter, Esc, F, Home, Insert, Left, Null, PageDown, PageUp, Right, Tab, Up},
  KeyEvent, KeyModifiers, MouseEventKind,
};
use futures::StreamExt;
use log::{Level, LevelFilter, debug, error, info, log_enabled, trace, warn};
use serde::{Deserialize, Serialize};
use tokio::{
  sync::{mpsc, oneshot},
  task::JoinHandle,
};

#[derive(Debug, Clone)]
pub enum Event<I> {
  Input(I),
  Paste(String),
  Resize,
  Tick,
  Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq)]
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
                  if let Some(Ok(event)) = maybe_event
                      && let Some(event) = translate(event)
                  {
                      _tx.send(event).unwrap_or_else(|e| warn!("Unable to send {:?} event", e.0));
                  }
              }
          }
        }
      });
    }

    Self { tx, rx, tick_rate, abort }
  }
}

/// Maps a crossterm event to the app's event, or None when the app ignores it.
fn translate(event: crossterm::event::Event) -> Option<Event<KeyCode>> {
  match event {
    crossterm::event::Event::Key(key) => {
      let key = match key.code {
        Backspace => match key.modifiers {
          KeyModifiers::CONTROL => KeyCode::CtrlBackspace,
          KeyModifiers::ALT => KeyCode::AltBackspace,
          _ => KeyCode::Backspace,
        },
        Delete => match key.modifiers {
          KeyModifiers::CONTROL => KeyCode::CtrlDelete,
          KeyModifiers::ALT => KeyCode::AltDelete,
          _ => KeyCode::Delete,
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
      Some(Event::Input(key))
    }
    crossterm::event::Event::Paste(paste) => Some(Event::Paste(paste)),
    // Translate scroll wheel events into synthetic Up/Down keypresses so the
    // existing arrow-key handlers in app.rs scroll the task list. Other mouse
    // event kinds (clicks, drags, motion) are intentionally ignored to avoid
    // interfering with terminal text selection.
    crossterm::event::Event::Mouse(mouse) => match mouse.kind {
      MouseEventKind::ScrollUp => Some(Event::Input(KeyCode::Up)),
      MouseEventKind::ScrollDown => Some(Event::Input(KeyCode::Down)),
      _ => None,
    },
    // A resize can leave the terminal grid out of step with ratatui's buffer.
    // tmux drops rows off the top when a pane shrinks, and a shrink followed by
    // a regrow between two draws ends at the size ratatui last saw, so its
    // autoresize never clears and the shifted rows stay on screen for good.
    crossterm::event::Event::Resize(_, _) => Some(Event::Resize),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use crossterm::event::{Event as CrosstermEvent, KeyCode as CrosstermKeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

  use super::*;

  fn mouse(kind: MouseEventKind) -> CrosstermEvent {
    CrosstermEvent::Mouse(MouseEvent {
      kind,
      column: 0,
      row: 0,
      modifiers: KeyModifiers::NONE,
    })
  }

  #[test]
  fn test_translate_forwards_resize() {
    assert!(matches!(translate(CrosstermEvent::Resize(62, 35)), Some(Event::Resize)));
  }

  #[test]
  fn test_translate_maps_keys() {
    let ctrl_r = CrosstermEvent::Key(KeyEvent::new(CrosstermKeyCode::Char('r'), KeyModifiers::CONTROL));
    assert!(matches!(translate(ctrl_r), Some(Event::Input(KeyCode::Ctrl('r')))));
    let enter = CrosstermEvent::Key(KeyEvent::new(CrosstermKeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(translate(enter), Some(Event::Input(KeyCode::Char('\n')))));
  }

  #[test]
  fn test_translate_maps_scroll_and_ignores_motion() {
    assert!(matches!(translate(mouse(MouseEventKind::ScrollUp)), Some(Event::Input(KeyCode::Up))));
    assert!(matches!(translate(mouse(MouseEventKind::ScrollDown)), Some(Event::Input(KeyCode::Down))));
    assert!(translate(mouse(MouseEventKind::Moved)).is_none());
  }
}
