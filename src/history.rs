use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::history::Direction;
use rustyline::history::History;
use std::fs::File;
use xdg::BaseDirectories;

pub struct HistoryContext {
    history: History,
    history_index: usize,
}

impl HistoryContext {
    pub fn new() -> Self {
        let history = History::new();
        Self {
            history,
            history_index: 0,
        }
    }

    pub fn load(&mut self, filename: &str) -> Result<()> {
        let d = BaseDirectories::with_prefix("taskwarrior-tui")?;
        if let Some(path) = d.find_config_file(filename) {
            self.history.load(&path)?;
            Ok(())
        } else {
            let path = d.place_config_file(filename)?;
            self.history.save(&path)?;
            Ok(())
        }
    }

    pub fn write(&mut self, filename: &str) -> Result<()> {
        let d = BaseDirectories::with_prefix("taskwarrior-tui")?;
        if let Some(path) = d.find_config_file(filename) {
            self.history.save(&path)?;
            Ok(())
        } else {
            let path = d.place_config_file(filename)?;
            self.history.save(&path)?;
            Ok(())
        }
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
            return Some(self.history.get(self.history_index).unwrap().clone());
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
}
