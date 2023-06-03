use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::StreamExt;
use log::warn;
use tokio::sync::mpsc;

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
  /// Terminal tick.
  Closed,
  /// Terminal tick.
  Tick,
  /// Key press.
  Key(KeyEvent),
  /// Mouse click/scroll.
  Mouse(MouseEvent),
  /// Terminal resize.
  Resize(u16, u16),
}

/// Terminal event handler.
#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
  pub sender: mpsc::UnboundedSender<Event>,
  pub receiver: mpsc::UnboundedReceiver<Event>,
  pub abort: mpsc::UnboundedSender<()>,
}

impl EventHandler {
  pub fn new(tick_rate: u64) -> Self {
    let tick_rate = if tick_rate == 0 { Some(Duration::from_millis(tick_rate)) } else { None };
    let (sender, receiver) = mpsc::unbounded_channel();
    let _sender = sender.clone();
    let (abort, mut abort_recv) = mpsc::unbounded_channel();

    let should_tick = tick_rate.is_some();
    let tick_rate = tick_rate.unwrap_or(std::time::Duration::from_millis(250));

    let mut reader = crossterm::event::EventStream::new();
    tokio::spawn(async move {
      loop {
        let delay = tokio::time::sleep(tick_rate);
        let event = reader.next();

        tokio::select! {
            _ = abort_recv.recv() => {
                _sender.send(Event::Closed).unwrap_or_else(|_| warn!("Unable to send Closed event"));
                _sender.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
                break;
            },
            _ = delay, if should_tick => {
                _sender.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
            },
            _ = _sender.closed() => break,
            maybe_event = event => {
                if let Some(Ok(CrosstermEvent::Key(key))) = maybe_event {
                    _sender.send(Event::Key(key)).unwrap_or_else(|_| warn!("Unable to send {:?} event", key));
                }
            }
        }
      }
    });

    Self { sender, receiver, abort }
  }

  pub async fn next(&mut self) -> Option<Event> {
    Some(self.receiver.recv().await?)
  }
}
