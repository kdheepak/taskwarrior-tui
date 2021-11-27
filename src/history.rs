use anyhow::{anyhow, Result};
use rustyline::error::ReadlineError;
use rustyline::history::Direction;
use rustyline::history::History;
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct HistoryContext {
    history: History,
    history_index: usize,
    config_path: PathBuf,
}

impl HistoryContext {
    pub fn new(filename: &str) -> Self {
        let history = History::new();

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
        self.history_index = self.history.len();
        log::debug!("Loading history of length {}", self.history.len());
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
        log::debug!(
            "Searching history for {:?} in direction {:?} with history index = {:?}",
            buf,
            dir,
            self.history_index()
        );
        if self.history.is_empty() {
            log::debug!("History is empty");
            return None;
        }
        if self.history_index == self.history.len().saturating_sub(1) && dir == Direction::Forward
            || self.history_index == 0 && dir == Direction::Reverse
        {
            log::debug!("No more history left to search");
            return None;
        }
        let history_index = match dir {
            Direction::Reverse => self.history_index.saturating_sub(1),
            Direction::Forward => self
                .history_index
                .saturating_add(1)
                .min(self.history_len().saturating_sub(1)),
        };
        log::debug!("Using history index = {} for searching", history_index);
        if let Some(history_index) = self.history.starts_with(buf, history_index, dir) {
            log::debug!("Found index {:?}", history_index);
            log::debug!("Previous index {:?}", self.history_index);
            self.history_index = history_index;
            Some(self.history.get(history_index).unwrap().clone())
        } else if buf.is_empty() {
            self.history_index = history_index;
            Some(self.history.get(history_index).unwrap().clone())
        } else {
            log::debug!("History index = {}. Found no match.", history_index);
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
