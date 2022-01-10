use anyhow::{anyhow, Result};
use rustyline::error::ReadlineError;
use rustyline::history::History;
use rustyline::history::SearchDirection;
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct HistoryContext {
    history: History,
    history_index: Option<usize>,
    data_path: PathBuf,
}

impl HistoryContext {
    pub fn new(filename: &str) -> Self {
        let history = History::new();

        let data_path = if let Ok(s) = std::env::var("TASKWARRIOR_TUI_DATA") {
            PathBuf::from(s)
        } else {
            dirs::data_dir()
                .map(|d| d.join("taskwarrior-tui"))
                .expect("Unable to create configuration directory for taskwarrior-tui")
        };

        std::fs::create_dir_all(&data_path)
            .unwrap_or_else(|_| panic!("Unable to create configuration directory in {:?}", &data_path));

        let data_path = data_path.join(filename);

        Self {
            history,
            history_index: None,
            data_path,
        }
    }

    pub fn load(&mut self) -> Result<()> {
        if self.data_path.exists() {
            self.history.load(&self.data_path)?;
        } else {
            self.history.save(&self.data_path)?;
        }
        self.history_index = None;
        log::debug!("Loading history of length {}", self.history.len());
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        self.history.save(&self.data_path)?;
        Ok(())
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn history_index(&self) -> Option<usize> {
        self.history_index
    }

    pub fn history_search(&mut self, buf: &str, dir: SearchDirection) -> Option<String> {
        log::debug!(
            "Searching history for {:?} in direction {:?} with history index = {:?} and history len = {:?}",
            buf,
            dir,
            self.history_index(),
            self.history.len(),
        );

        if self.history.is_empty() {
            log::debug!("History is empty");
            return None;
        }

        let history_index = if self.history_index().is_none() {
            log::debug!("History index is none");
            match dir {
                SearchDirection::Forward => return None,
                SearchDirection::Reverse => self.history_index = Some(self.history_len().saturating_sub(1)),
            }
            self.history_index.unwrap()
        } else {
            let hi = self.history_index().unwrap();

            if hi == self.history.len().saturating_sub(1) && dir == SearchDirection::Forward
                || hi == 0 && dir == SearchDirection::Reverse
            {
                return None;
            }

            match dir {
                SearchDirection::Reverse => hi.saturating_sub(1),
                SearchDirection::Forward => hi.saturating_add(1).min(self.history_len().saturating_sub(1)),
            }
        };

        log::debug!("Using history index = {} for searching", history_index);
        return if let Some(history_index) = self.history.starts_with(buf, history_index, dir) {
            log::debug!("Found index {:?}", history_index);
            log::debug!("Previous index {:?}", self.history_index);
            self.history_index = Some(history_index.idx);
            Some(history_index.entry.to_string())
        } else if buf.is_empty() {
            self.history_index = Some(history_index);
            self.history.get(history_index).cloned()
        } else {
            log::debug!("History index = {}. Found no match.", history_index);
            None
        };
    }

    pub fn add(&mut self, buf: &str) {
        if self.history.add(buf) {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        self.history_index = None
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}
