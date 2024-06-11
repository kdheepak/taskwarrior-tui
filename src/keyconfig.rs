use std::{collections::HashSet, error::Error, hash::Hash};

use anyhow::{anyhow, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::event::KeyCode;

static KEYCONFIG_PREFIX: &str = "uda.taskwarrior-tui.keyconfig";

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyConfig {
  pub quit: KeyCode,
  pub refresh: KeyCode,
  pub go_to_bottom: KeyCode,
  pub go_to_top: KeyCode,
  pub down: KeyCode,
  pub up: KeyCode,
  pub page_down: KeyCode,
  pub page_up: KeyCode,
  pub delete: KeyCode,
  pub done: KeyCode,
  pub start_stop: KeyCode,
  pub quick_tag: KeyCode,
  pub select: KeyCode,
  pub select_all: KeyCode,
  pub undo: KeyCode,
  pub edit: KeyCode,
  pub duplicate: KeyCode,
  pub modify: KeyCode,
  pub shell: KeyCode,
  pub log: KeyCode,
  pub add: KeyCode,
  pub annotate: KeyCode,
  pub help: KeyCode,
  pub filter: KeyCode,
  pub zoom: KeyCode,
  pub context_menu: KeyCode,
  pub next_tab: KeyCode,
  pub previous_tab: KeyCode,
  pub priority_h: KeyCode,
  pub priority_m: KeyCode,
  pub priority_l: KeyCode,
  pub priority_n: KeyCode,
  pub shortcut0: KeyCode,
  pub shortcut1: KeyCode,
  pub shortcut2: KeyCode,
  pub shortcut3: KeyCode,
  pub shortcut4: KeyCode,
  pub shortcut5: KeyCode,
  pub shortcut6: KeyCode,
  pub shortcut7: KeyCode,
  pub shortcut8: KeyCode,
  pub shortcut9: KeyCode,
}

impl Default for KeyConfig {
  fn default() -> Self {
    Self {
      quit: KeyCode::Char('q'),
      refresh: KeyCode::Char('r'),
      go_to_bottom: KeyCode::Char('G'),
      go_to_top: KeyCode::Char('g'),
      down: KeyCode::Char('j'),
      up: KeyCode::Char('k'),
      page_down: KeyCode::Char('J'),
      page_up: KeyCode::Char('K'),
      delete: KeyCode::Char('x'),
      done: KeyCode::Char('d'),
      start_stop: KeyCode::Char('s'),
      quick_tag: KeyCode::Char('t'),
      select: KeyCode::Char('v'),
      select_all: KeyCode::Char('V'),
      undo: KeyCode::Char('u'),
      edit: KeyCode::Char('e'),
      duplicate: KeyCode::Char('y'),
      modify: KeyCode::Char('m'),
      shell: KeyCode::Char('!'),
      log: KeyCode::Char('l'),
      add: KeyCode::Char('a'),
      annotate: KeyCode::Char('A'),
      help: KeyCode::Char('?'),
      filter: KeyCode::Char('/'),
      zoom: KeyCode::Char('z'),
      context_menu: KeyCode::Char('c'),
      next_tab: KeyCode::Char(']'),
      previous_tab: KeyCode::Char('['),
      priority_h: KeyCode::Char('H'),
      priority_m: KeyCode::Char('M'),
      priority_l: KeyCode::Char('L'),
      priority_n: KeyCode::Char('N'),
      shortcut0: KeyCode::Char('0'),
      shortcut1: KeyCode::Char('1'),
      shortcut2: KeyCode::Char('2'),
      shortcut3: KeyCode::Char('3'),
      shortcut4: KeyCode::Char('4'),
      shortcut5: KeyCode::Char('5'),
      shortcut6: KeyCode::Char('6'),
      shortcut7: KeyCode::Char('7'),
      shortcut8: KeyCode::Char('8'),
      shortcut9: KeyCode::Char('9'),
    }
  }
}

impl KeyConfig {
  pub fn new(data: &str) -> Result<Self> {
    let mut kc = Self::default();
    kc.update(data)?;
    Ok(kc)
  }

  pub fn update(&mut self, data: &str) -> Result<()> {
    // Set key to value in config file, if config file contains it
    let update_key = | key: &mut KeyCode, key_name: &str | {
      let config_name = format!("{KEYCONFIG_PREFIX}.{key_name}");
      let key_from_config = Self::get_config(&config_name, data);

      if let Some(new_key) = key_from_config {
        *key = new_key;
      }
    };

    update_key(&mut self.quit, "quit");
    update_key(&mut self.refresh, "refresh");
    update_key(&mut self.go_to_bottom, "go-to-bottom");
    update_key(&mut self.go_to_top, "go-to-top");
    update_key(&mut self.down, "down");
    update_key(&mut self.up, "up");
    update_key(&mut self.page_down, "page-down");
    update_key(&mut self.page_up, "page-up");
    update_key(&mut self.delete, "delete");
    update_key(&mut self.done, "done");
    update_key(&mut self.start_stop, "start-stop");
    update_key(&mut self.quick_tag, "quick-tag");
    update_key(&mut self.select, "select");
    update_key(&mut self.select_all, "select-all");
    update_key(&mut self.undo, "undo");
    update_key(&mut self.edit, "edit");
    update_key(&mut self.duplicate, "duplicate");
    update_key(&mut self.modify, "modify");
    update_key(&mut self.shell, "shell");
    update_key(&mut self.log, "log");
    update_key(&mut self.add, "add");
    update_key(&mut self.annotate, "annotate");
    update_key(&mut self.filter, "filter");
    update_key(&mut self.zoom, "zoom");
    update_key(&mut self.context_menu, "context-menu");
    update_key(&mut self.next_tab, "next-tab");
    update_key(&mut self.previous_tab, "previous-tab");
    update_key(&mut self.shortcut0, "shortcut0");
    update_key(&mut self.shortcut1, "shortcut1");
    update_key(&mut self.shortcut2, "shortcut2");
    update_key(&mut self.shortcut3, "shortcut3");
    update_key(&mut self.shortcut4, "shortcut4");
    update_key(&mut self.shortcut5, "shortcut5");
    update_key(&mut self.shortcut6, "shortcut6");
    update_key(&mut self.shortcut7, "shortcut7");
    update_key(&mut self.shortcut8, "shortcut8");
    update_key(&mut self.shortcut9, "shortcut9");

    self.check()
  }

  pub fn check(&self) -> Result<()> {
    let mut elements = vec![
      &self.quit,
      &self.refresh,
      &self.go_to_bottom,
      &self.go_to_top,
      &self.down,
      &self.up,
      &self.page_down,
      &self.page_up,
      &self.delete,
      &self.done,
      &self.select,
      &self.select_all,
      &self.start_stop,
      &self.quick_tag,
      &self.undo,
      &self.edit,
      &self.duplicate,
      &self.modify,
      &self.shell,
      &self.log,
      &self.add,
      &self.annotate,
      &self.help,
      &self.filter,
      &self.zoom,
      &self.context_menu,
      &self.next_tab,
      &self.previous_tab,
    ];
    let l = elements.len();
    elements.dedup();
    if l == elements.len() {
      Ok(())
    } else {
      Err(anyhow!("Duplicate keys found in key config"))
    }
  }

  fn get_config(config: &str, data: &str) -> Option<KeyCode> {
    for line in data.split('\n') {
      if line.starts_with(config) {
        let line = line.trim_start_matches(config).trim_start().trim_end().to_string();
        if has_just_one_char(&line) {
          return Some(KeyCode::Char(line.chars().next().unwrap()));
        } else {
          error!("Found multiple characters in {} for {}", line, config);
        }
      } else if line.starts_with(&config.replace('-', "_")) {
        let line = line.trim_start_matches(&config.replace('-', "_")).trim_start().trim_end().to_string();
        if has_just_one_char(&line) {
          return Some(KeyCode::Char(line.chars().next().unwrap()));
        } else {
          error!("Found multiple characters in {} for {}", line, config);
        }
      }
    }
    None
  }
}

fn has_just_one_char(s: &str) -> bool {
  let mut chars = s.chars();
  chars.next().is_some() && chars.next().is_none()
}

#[cfg(test)]
mod tests {
  use super::*;
}
