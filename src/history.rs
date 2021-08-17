use anyhow::{anyhow, Result};
use rustyline::error::ReadlineError;
use rustyline::history::Direction;
use rustyline::history::History;
use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
use std::env;

pub struct HistoryContext {
    history: History,
    history_index: usize,
    config_path: PathBuf,
}

impl HistoryContext {
    pub fn new(filename: &str) -> Self {
        let history = History::new();

        #[cfg(target_os = "macos")]
        let config_dir_op = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| dirs::home_dir().map(|d| d.join(".config")));

        #[cfg(not(target_os = "macos"))]
        let config_dir_op = dirs::config_dir();

        let config_path = config_dir_op.map(|d| d.join("taskwarrior-tui")).unwrap();

        std::fs::create_dir_all(&config_path).unwrap();

        let config_path = config_path.join(filename);

        Self {
            history,
            history_index: 0,
            config_path,
        }
    }

    pub fn load(&mut self) -> Result<()> {
        if self.config_path.exists() {
            self.history.load(&self.config_path)?;
        } else {
            self.history.save(&self.config_path)?;
        }
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        self.history.save(&self.config_path)?;
        Ok(())
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn history_index(&self) -> usize {
        self.history_index
    }

    pub fn history_search(&mut self, buf: &str, dir: Direction) -> Option<String> {
        if self.history.is_empty() {
            return None;
        }
        if self.history_index == self.history.len().saturating_sub(1) && dir == Direction::Forward
            || self.history_index == 0 && dir == Direction::Reverse
        {
            return None;
        }
        let history_index = match dir {
            Direction::Reverse => self.history_index - 1,
            Direction::Forward => self.history_index + 1,
        };
        if let Some(history_index) = self.history.starts_with(buf, history_index, dir) {
            self.history_index = history_index;
            Some(self.history.get(history_index).unwrap().clone())
        } else if buf.is_empty() {
            self.history_index = history_index;
            Some(self.history.get(history_index).unwrap().clone())
        } else {
            None
        }
    }

    pub fn add(&mut self, buf: &str) {
        if self.history.add(buf) {
            self.history_index = self.history.len() - 1;
        }
    }

    pub fn last(&mut self) {
        self.history_index = self.history.len().saturating_sub(1);
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}
