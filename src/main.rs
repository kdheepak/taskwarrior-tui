#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod util;

#[allow(dead_code)]
mod app;

use crate::util::{Event, Events, Config};
use std::{error::Error, io};
use termion::{
    event::Key,
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{backend::TermionBackend, Terminal};
use unicode_width::UnicodeWidthStr;
use std::time::Duration;

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
    let events = Events::with_config(Config{
        exit_key: Key::Char('q'),
        tick_rate: Duration::from_secs(5),
    });

    let mut app = App::new();
    app.next();

    loop {
        terminal.draw(|mut frame| app.draw(&mut frame)).unwrap();

        // Handle input
        if let Event::Input(input) = events.next()? {
            match app.input_mode {
                InputMode::Normal => match input {
                    Key::Ctrl('c') | Key::Char('q') => break,
                    Key::Char('r')                  => app.update(),
                    Key::Down | Key::Char('j')      => app.next(),
                    Key::Up | Key::Char('k')        => app.previous(),
                    Key::Char('i') => {
                        app.input_mode = InputMode::Command;
                    }
                    _ => {},
                },
                InputMode::Command => match input {
                     Key::Char('\n') | Key::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    Key::Char(c) => {
                        app.filter.push(c);
                        app.update();
                    }
                    Key::Backspace => {
                        app.filter.pop();
                        app.update();
                    }
                    _ => {}
                },
            }
        }
    }

    Ok(())
}
