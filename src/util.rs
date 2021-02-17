use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{sync::mpsc, thread, time::Duration, time::Instant};

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

pub fn setup_terminal() -> Terminal<CrosstermBackend<io::Stdout>> {
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    execute!(stdout, Clear(ClearType::All)).unwrap();
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).unwrap()
}

pub fn destruct_terminal() {
    disable_raw_mode().unwrap();
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
    execute!(io::stdout(), cursor::Show).unwrap();
}

pub struct Events {
    pub rx: mpsc::Receiver<Event<Key>>,
    pub tx: mpsc::Sender<Event<Key>>,
    pub pause_stdin: Arc<AtomicBool>,
    pub handle: Option<thread::JoinHandle<()>>,
}

impl Events {
    #[cfg(feature = "crossterm")]
    pub fn with_config(config: EventConfig) -> Events {
        use crossterm::event::{KeyCode::*, KeyModifiers};
        let (tx, rx) = mpsc::channel();
        let pause_stdin = Arc::new(AtomicBool::new(false));
        let tick_rate = config.tick_rate;
        let handle = Some({
            let tx = tx.clone();
            let pause_stdin = pause_stdin.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {

                    if pause_stdin.load(Ordering::SeqCst) {
                        thread::sleep(Duration::from_millis(250));
                        thread::park();
                        continue;
                    }

                    let timeout = Duration::from_millis(25)
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or_else(|| Duration::from_millis(10));

                    if event::poll(timeout).unwrap() {
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

                    if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                        last_tick = Instant::now();
                    }
                }
            })
        });
        Events {
            rx,
            tx,
            pause_stdin,
            handle,
        }
    }

    /// Attempts to read an event.
    /// This function will block the current thread.
    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn pause_event_loop(&self) {
        self.pause_stdin.store(true, Ordering::SeqCst);
        thread::yield_now();
        while !self.pause_stdin.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn resume_event_loop(&self) {
        self.pause_stdin.store(false, Ordering::SeqCst);
        thread::yield_now();
        while self.pause_stdin.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(50));
        }
        self.handle.as_ref().unwrap().thread().unpark();
    }

    pub fn pause_key_capture(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        self.pause_event_loop();
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        terminal.show_cursor().unwrap();
    }

    pub fn resume_key_capture(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();
        enable_raw_mode().unwrap();
        self.resume_event_loop();
        terminal.resize(terminal.size().unwrap()).unwrap();
    }
}
