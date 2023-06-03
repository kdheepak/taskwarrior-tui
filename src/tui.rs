use std::io;

use anyhow::Result;
use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::Backend, Terminal};

use crate::{app::App, event::EventHandler, ui};

#[derive(Debug)]
pub struct Tui<B: Backend> {
  terminal: Terminal<B>,
  pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
  pub fn new(terminal: Terminal<B>, app: &mut App) -> Self {
    let events = EventHandler::new(app.config.uda_tick_rate);
    Self { terminal, events }
  }

  pub fn init(&mut self) -> Result<()> {
    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
    self.terminal.hide_cursor()?;
    self.terminal.clear()?;
    Ok(())
  }

  pub fn draw(&mut self, app: &mut App) -> Result<()> {
    self.terminal.draw(|frame| ui::render(frame, app))?;
    Ok(())
  }

  pub fn exit(&mut self) -> Result<()> {
    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
    self.terminal.show_cursor()?;
    Ok(())
  }
}
