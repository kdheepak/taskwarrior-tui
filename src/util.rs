#[cfg(feature = "crossterm")]
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(feature = "crossterm")]
use tui::{backend::CrosstermBackend, Terminal};

#[cfg(all(feature = "termion", not(feature = "crossterm")))]
use termion::{
    event,
    input::{MouseTerminal, TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen::{AlternateScreen,ToMainScreen, ToAlternateScreen},
};
#[cfg(all(feature = "termion", not(feature = "crossterm")))]
use tui::{backend::TermionBackend, Terminal};


use std::io::{self, Write};
use std::{sync::mpsc, thread, time::Duration};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Debug, Clone, Copy)]
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

#[cfg(feature = "crossterm")]
pub fn setup_terminal() -> Terminal<CrosstermBackend<io::Stdout>> {
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).unwrap()
}

#[cfg(feature = "crossterm")]
pub fn destruct_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) {
    disable_raw_mode().unwrap();
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
    terminal.show_cursor().unwrap();
}

#[cfg(all(feature = "termion", not(feature = "crossterm")))]
pub fn setup_terminal(
) -> Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>> {
    let raw_stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(raw_stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    Terminal::new(backend).unwrap()
}

#[cfg(all(feature = "termion", not(feature = "crossterm")))]
pub fn destruct_terminal(
    terminal: Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>>,
) {
}

pub struct Events {
    pub rx: mpsc::Receiver<Event<Key>>,
    pub tx: mpsc::Sender<Event<Key>>,
    pub pause_stdin: Arc<Mutex<bool>>,
}

impl Events {

    #[cfg(feature = "crossterm")]
    pub fn with_config(config: EventConfig) -> Events {
        use crossterm::event::{KeyCode::*, KeyModifiers};
        let (tx, rx) = mpsc::channel();
        let pause_stdin = Arc::new(Mutex::new(false));
        let pause_stdin = pause_stdin.clone();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                loop {
                    // poll for tick rate duration, if no event, sent tick event.
                    if let event::Event::Key(key) = event::read().unwrap() {
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
                            Tab => Key::Char('\t'),
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
                        tx.send(Event::Input(key)).unwrap();
                    }
                }
            })
        };
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };

        Events { rx, tx, pause_stdin }
    }

    #[cfg(all(feature = "termion", not(feature = "crossterm")))]
    pub fn with_config(config: EventConfig) -> Events {
        use termion::event::Key::*;
        let (tx, rx) = mpsc::channel();
        let pause_stdin = Arc::new(Mutex::new(false));
        let input_handle = {
            let tx = tx.clone();
            let pause_stdin = pause_stdin.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    while *pause_stdin.lock().unwrap() {
                        thread::sleep(config.tick_rate);
                    }
                    if let Ok(key) = evt {
                        let key = match key {
                            Backspace => Key::Backspace,
                            Left => Key::Left,
                            Right => Key::Right,
                            Up => Key::Up,
                            Down => Key::Down,
                            Home => Key::Home,
                            End => Key::End,
                            PageUp => Key::PageUp,
                            PageDown => Key::PageDown,
                            BackTab => Key::BackTab,
                            Delete => Key::Delete,
                            Insert => Key::Insert,
                            F(c) => Key::F(c),
                            Char(c) => Key::Char(c),
                            Alt(c) => Key::Alt(c),
                            Ctrl(c) => Key::Ctrl(c),
                            Null => Key::Null,
                            Esc => Key::Esc,
                            _ => Key::Null,
                        };
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                    }
                }
            })
        };
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };
        Events { rx, tx, pause_stdin }
    }

    /// Attempts to read an event.
    /// This function will block the current thread.
    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    #[cfg(all(feature = "termion", not(feature = "crossterm")))]
    pub fn pause_event_loop(&self, terminal: & mut Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>>) {
        *self.pause_stdin.lock().unwrap() = true;
        std::thread::sleep(std::time::Duration::from_millis(50));
        write!(terminal.backend_mut(), "{}", ToMainScreen).unwrap();
    }

    #[cfg(all(feature = "termion", not(feature = "crossterm")))]
    pub fn resume_event_loop(&self, terminal: & mut Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>>) {
        write!(terminal.backend_mut(), "{}", ToAlternateScreen).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        *self.pause_stdin.lock().unwrap() = false;
        terminal.resize(terminal.size().unwrap()).unwrap();
    }

    #[cfg(feature = "crossterm")]
    pub fn pause_event_loop(&self, terminal: & mut Terminal<CrosstermBackend<io::Stdout>>) {
        *self.pause_stdin.lock().unwrap() = true;
        std::thread::sleep(std::time::Duration::from_millis(50));
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        terminal.show_cursor().unwrap();
    }

    #[cfg(feature = "crossterm")]
    pub fn resume_event_loop(&self, terminal: & mut Terminal<CrosstermBackend<io::Stdout>>) {
        enable_raw_mode().unwrap();
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        *self.pause_stdin.lock().unwrap() = false;
        terminal.resize(terminal.size().unwrap()).unwrap();
    }
}
