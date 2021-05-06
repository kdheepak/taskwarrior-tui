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
use unicode_width::UnicodeWidthStr;

pub fn get_start_word_under_cursor(line: &str, cursor_pos: usize) -> usize {
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
    pub candidates: Vec<String>,
    pub completer: rustyline::completion::FilenameCompleter,
}

impl Completer for TaskwarriorTuiCompletionHelper {
    type Candidate = Pair;

    fn complete(&self, word: &str, pos: usize, _ctx: &Context) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let candidates: Vec<Pair> = self
            .candidates
            .iter()
            .filter_map(|candidate| {
                if candidate.starts_with(&word[..pos]) {
                    Some(Pair {
                        display: candidate.clone(),
                        replacement: candidate[pos..].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok((pos, candidates))
    }
}

pub struct CompletionList {
    pub state: ListState,
    pub input: String,
    pub pos: usize,
    pub helper: TaskwarriorTuiCompletionHelper,
}

impl CompletionList {
    pub fn new() -> CompletionList {
        let completer = FilenameCompleter::new();
        CompletionList {
            state: ListState::default(),
            input: String::new(),
            pos: 0,
            helper: TaskwarriorTuiCompletionHelper {
                candidates: vec![],
                completer,
            },
        }
    }

    pub fn with_items(items: Vec<String>) -> CompletionList {
        let completer = FilenameCompleter::new();
        let mut candidates = vec![];
        for i in items {
            if !candidates.contains(&i) {
                candidates.push(i);
            }
        }
        CompletionList {
            state: ListState::default(),
            input: String::new(),
            pos: 0,
            helper: TaskwarriorTuiCompletionHelper { candidates, completer },
        }
    }

    pub fn insert(&mut self, item: String) {
        if !self.helper.candidates.contains(&item) {
            self.helper.candidates.push(item);
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.candidates().len() - 1 {
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
                    self.candidates().len() - 1
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
        self.helper.candidates.clear();
        self.state.select(None);
    }

    pub fn len(&self) -> usize {
        self.candidates().len()
    }

    pub fn max_width(&self) -> Option<usize> {
        self.candidates().iter().map(|p| p.display.width() + 4).max()
    }

    pub fn get(&self, i: usize) -> Option<String> {
        let candidates = self.candidates();
        if i < candidates.len() {
            Some(candidates[i].replacement.clone())
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
        self.candidates().is_empty()
    }

    pub fn candidates(&self) -> Vec<Pair> {
        let hist = rustyline::history::History::new();
        let ctx = rustyline::Context::new(&hist);
        let (pos, candidates) = self.helper.complete(&self.input, self.pos, &ctx).unwrap();
        candidates
    }

    pub fn input(&mut self, input: String) {
        self.input = input;
        self.pos = self.input.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion() {
        let mut completion_list = CompletionList::new();

        completion_list.insert("+test".to_string());
        completion_list.insert("+shortcut".to_string());
        completion_list.insert("project:color".to_string());
        completion_list.insert("due:'2021-04-07T00:00:00'".to_string());

        completion_list.input("due:".to_string());

        for p in completion_list.candidates().iter() {
            dbg!(format!("{:?}", p.display));
            dbg!(format!("{:?}", p.replacement));
        }
    }
}
