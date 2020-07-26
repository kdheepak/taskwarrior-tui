#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod util;

#[allow(dead_code)]
mod app;

use crate::util::{Config, Event, Events};
use std::time::Duration;
use std::{error::Error, io};
use termion::{
    event::Key,
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{backend::TermionBackend, Terminal};
use unicode_width::UnicodeWidthStr;

use app::App;
use app::InputMode;

type B = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>;

fn setup_terminal() -> Result<Terminal<B>, io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    Terminal::new(backend)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let mut terminal = setup_terminal()?;

    // Setup event handlers
    let mut events = Events::with_config(Config {
        exit_key: Key::Char('q'),
        tick_rate: Duration::from_secs(1),
    });
    events.disable_exit_key();

    let mut app = App::new();
    app.next();

    loop {
        terminal.draw(|mut frame| app.draw(&mut frame)).unwrap();

        // Handle input
        match events.next()? {
            Event::Input(input) => app.handle_input(input),
            Event::Tick => app.handle_tick(),
        }

        if app.should_quit {
            break
        }
    }

    Ok(())
}
