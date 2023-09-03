use std::ops::{Deref, DerefMut};

use color_eyre::eyre::Result;
use crossterm::{
  cursor,
  event::{Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent},
  terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use serde_derive::{Deserialize, Serialize};
use tokio::{
  sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

pub type CrosstermFrame<'a> = ratatui::Frame<'a, Backend<std::io::Stderr>>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Event {
  Quit,
  Error,
  Closed,
  Tick,
  Key(KeyEvent),
  Mouse(MouseEvent),
  Resize(u16, u16),
}

pub struct Tui {
  pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
  pub task: JoinHandle<()>,
  pub cancellation_token: CancellationToken,
  pub event_rx: UnboundedReceiver<Event>,
  pub event_tx: UnboundedSender<Event>,
  pub tick_rate: usize,
}

impl Tui {
  pub fn new(tick_rate: usize) -> Result<Self> {
    let terminal = ratatui::Terminal::new(Backend::new(std::io::stderr()))?;
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let cancellation_token = CancellationToken::new();
    let task = tokio::spawn(async {});
    Ok(Self {
      terminal,
      task,
      cancellation_token,
      event_rx,
      event_tx,
      tick_rate,
    })
  }

  pub fn start(&mut self) {
    let tick_rate = std::time::Duration::from_millis(self.tick_rate as u64);
    self.cancellation_token.cancel();
    self.cancellation_token = CancellationToken::new();
    let _cancellation_token = self.cancellation_token.clone();
    let _event_tx = self.event_tx.clone();
    self.task = tokio::spawn(async move {
      let mut reader = crossterm::event::EventStream::new();
      let mut interval = tokio::time::interval(tick_rate);
      loop {
        let delay = interval.tick();
        let crossterm_event = reader.next().fuse();
        tokio::select! {
          _ = _cancellation_token.cancelled() => {
            break;
          }
          maybe_event = crossterm_event => {
            match maybe_event {
              Some(Ok(evt)) => {
                match evt {
                  CrosstermEvent::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                      _event_tx.send(Event::Key(key)).unwrap();
                    }
                  },
                  CrosstermEvent::Resize(x, y) => {
                    _event_tx.send(Event::Resize(x, y)).unwrap();
                  },
                  _ => {},
                }
              }
              Some(Err(_)) => {
                _event_tx.send(Event::Error).unwrap();
              }
              None => {},
            }
          },
          _ = delay => {
              _event_tx.send(Event::Tick).unwrap();
          },
        }
      }
    });
  }

  pub fn enter(&mut self) -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)?;
    self.start();
    Ok(())
  }

  pub fn exit(&self) -> Result<()> {
    crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
    crossterm::terminal::disable_raw_mode()?;
    self.cancellation_token.cancel();
    Ok(())
  }

  pub fn suspend(&self) -> Result<()> {
    self.exit()?;
    #[cfg(not(windows))]
    signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
    Ok(())
  }

  pub fn resume(&mut self) -> Result<()> {
    self.enter()?;
    Ok(())
  }

  pub async fn next(&mut self) -> Option<Event> {
    self.event_rx.recv().await
  }
}

impl Deref for Tui {
  type Target = ratatui::Terminal<Backend<std::io::Stderr>>;

  fn deref(&self) -> &Self::Target {
    &self.terminal
  }
}

impl DerefMut for Tui {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.terminal
  }
}

impl Drop for Tui {
  fn drop(&mut self) {
    self.exit().unwrap();
  }
}
