use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
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
    pub candidates: Vec<(String, String)>,
    pub context: String,
    pub input: String,
}

impl Completer for TaskwarriorTuiCompletionHelper {
    type Candidate = Pair;

    fn complete(&self, word: &str, pos: usize, _ctx: &Context) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let candidates: Vec<Pair> = self
            .candidates
            .iter()
            .filter_map(|(context, candidate)| {
                if context == &self.context && candidate.starts_with(&word[..pos]) && !self.input.contains(candidate) {
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
    pub current: String,
    pub pos: usize,
    pub helper: TaskwarriorTuiCompletionHelper,
}

impl CompletionList {
    pub fn new() -> CompletionList {
        CompletionList {
            state: ListState::default(),
            current: String::new(),
            pos: 0,
            helper: TaskwarriorTuiCompletionHelper {
                candidates: vec![],
                context: String::new(),
                input: String::new(),
            },
        }
    }

    pub fn with_items(items: Vec<(String, String)>) -> CompletionList {
        let mut candidates = vec![];
        for i in items {
            if !candidates.contains(&i) {
                candidates.push(i);
            }
        }
        let context = String::new();
        let input = String::new();
        CompletionList {
            state: ListState::default(),
            current: String::new(),
            pos: 0,
            helper: TaskwarriorTuiCompletionHelper {
                candidates,
                context,
                input,
            },
        }
    }

    pub fn insert(&mut self, item: (String, String)) {
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
        self.state.selected().and_then(|i| self.get(i))
    }

    pub fn is_empty(&self) -> bool {
        self.candidates().is_empty()
    }

    pub fn candidates(&self) -> Vec<Pair> {
        let hist = rustyline::history::History::new();
        let ctx = rustyline::Context::new(&hist);
        let (pos, candidates) = self.helper.complete(&self.current, self.pos, &ctx).unwrap();
        candidates
    }

    pub fn input(&mut self, current: String, i: String) {
        self.helper.input = i;
        if current.contains('.') && current.contains(':') {
            self.current = current.split_once(':').unwrap().1.to_string();
            self.helper.context = current.split_once('.').unwrap().0.to_string();
        } else if current.contains('.') {
            self.current = format!(".{}", current.split_once('.').unwrap().1);
            self.helper.context = "modifier".to_string();
        } else if current.contains(':') {
            self.current = current.split_once(':').unwrap().1.to_string();
            self.helper.context = current.split_once(':').unwrap().0.to_string();
        } else if current.contains('+') {
            self.current = format!("+{}", current.split_once('+').unwrap().1);
            self.helper.context = "+".to_string();
        } else {
            self.current = current;
            self.helper.context = "attribute".to_string();
        }
        self.pos = self.current.len();
    }
}
