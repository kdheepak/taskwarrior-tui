#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod app;
mod calendar;
mod config;
mod context;
mod help;
mod keyconfig;
mod line_buffer;
mod table;
mod task_report;
mod util;

use crate::util::{destruct_terminal, setup_terminal, Event, EventConfig, Events};
use anyhow::Result;
use clap::{App, Arg};
use std::env;
use std::error::Error;
use std::io::Write;
use std::panic;
use std::time::Duration;

use async_std::prelude::*;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use futures::join;
use futures::stream::{FuturesUnordered, StreamExt};

use crate::util::Key;
use app::{AppMode, TaskwarriorTuiApp};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> Result<()> {
    better_panic::install();
    let matches = App::new(APP_NAME)
        .version(APP_VERSION)
        .author("Dheepak Krishnamurthy <@kdheepak>")
        .about("A taskwarrior terminal user interface")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config = matches.value_of("config").unwrap_or("~/.taskrc");
    task::block_on(tui_main(config))
}

async fn tui_main(_config: &str) -> Result<()> {
    // Terminal initialization
    let terminal = setup_terminal();

    panic::set_hook(Box::new(|panic_info| {
        destruct_terminal();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));

    // Setup event handlers
    let events = Events::with_config(EventConfig {
        tick_rate: Duration::from_millis(250),
    });

    let maybeapp = TaskwarriorTuiApp::new().await;
    match maybeapp {
        Ok(app) => {
            let app = Arc::new(Mutex::new(app));
            let terminal = Arc::new(Mutex::new(terminal));
            loop {
                let handle = {
                    let app = app.clone();
                    let terminal = terminal.clone();
                    task::spawn_local(async move {
                        let mut t = terminal.lock().await;
                        app.lock().await.render(&mut t).await.unwrap();
                    })
                };
                // Handle input
                match events.next().await? {
                    Event::Input(input) => {
                        let mut t = terminal.lock().await;
                        let r = app.lock().await.handle_input(input, &mut t, &events).await;
                        if r.is_err() {
                            destruct_terminal();
                            return r;
                        }
                    }
                    Event::Tick => {
                        let r = app.lock().await.update(false).await;
                        if r.is_err() {
                            destruct_terminal();
                            return r;
                        }
                    }
                }

                if app.lock().await.should_quit {
                    destruct_terminal();
                    break;
                }
            }

            Ok(())
        }
        Err(e) => {
            destruct_terminal();
            Err(e)
        }
    }
}
