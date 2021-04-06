use std::{error::Error, io};
use tui::{
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::Hinter;
use rustyline::line_buffer::LineBuffer;
use rustyline::Context;
use rustyline_derive::Helper;

use unicode_segmentation::Graphemes;
use unicode_segmentation::UnicodeSegmentation;

// find the beginning of the word in line which is currently under the cursor,
// whose position is cursor_pos.
//
fn get_start_word_under_cursor(line: &str, cursor_pos: usize) -> usize {
    let mut chars = line[..cursor_pos].chars();
    let mut res = cursor_pos;
    while let Some(c) = chars.next_back() {
        if c == ' ' || c == '(' || c == ')' {
            break;
        }
        res -= c.len_utf8();
    }
    // if iter == None, res == 0.
    res
}

pub struct TaskwarriorTuiCompletionHelper {
    hints: Vec<String>,
}

impl Completer for TaskwarriorTuiCompletionHelper {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let mut candidates: Vec<String> = self
            .hints
            .iter()
            .filter_map(|hint| {
                if pos > 0 && hint.starts_with(&line[..pos]) && !hint[pos..].contains(" ") {
                    Some(hint[pos..].to_owned())
                } else {
                    None
                }
            })
            .collect();
        candidates.sort();
        Ok((pos, candidates))
    }
}

pub struct CompletionList {
    pub state: ListState,
    pub helper: TaskwarriorTuiCompletionHelper,
}

impl CompletionList {
    pub fn new() -> CompletionList {
        CompletionList {
            state: ListState::default(),
            helper: TaskwarriorTuiCompletionHelper { hints: vec![] },
        }
    }

    pub fn with_items(items: Vec<String>) -> CompletionList {
        let mut hints = vec![];
        for i in items {
            if !hints.contains(&i) {
                hints.push(i);
            }
        }
        CompletionList {
            state: ListState::default(),
            helper: TaskwarriorTuiCompletionHelper { hints },
        }
    }

    pub fn insert(&mut self, item: String) {
        if !self.helper.hints.contains(&item) {
            self.helper.hints.push(item);
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.helper.hints.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.helper.hints.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn clear(&mut self) {
        self.helper.hints.clear();
        self.state.select(None);
    }

    pub fn len(&self) -> usize {
        self.helper.hints.iter().count()
    }

    pub fn max_width(&self) -> Option<usize> {
        self.helper.hints.iter().map(|s| s.graphemes(true).count() + 4).max()
    }

    pub fn get(&self, i: usize) -> Option<String> {
        if i < self.helper.hints.len() {
            Some(self.helper.hints[i].clone())
        } else {
            None
        }
    }

    pub fn selected(&self) -> Option<String> {
        if let Some(i) = self.state.selected() {
            self.get(i)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.helper.hints.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<String> {
        self.helper.hints.iter()
    }
}
