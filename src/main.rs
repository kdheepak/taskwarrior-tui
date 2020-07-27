#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod util;

#[allow(dead_code)]
mod app;

use crate::util::{destruct_terminal, setup_terminal, Event, EventConfig, Events};
use std::env;
use std::error::Error;
use std::io;
use std::io::{stdout, Write};
use std::process::Command;
use std::time::{Duration, Instant};
use tui::backend::Backend;
use unicode_width::UnicodeWidthStr;

use app::App;

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let mut terminal = setup_terminal();
    terminal.clear()?;

    // Setup event handlers
    let events = Events::with_config(EventConfig {
        tick_rate: Duration::from_millis(250),
    });

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
            destruct_terminal(terminal);
            break;
        }
    }
    Ok(())
}
