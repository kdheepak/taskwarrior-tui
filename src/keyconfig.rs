use std::{collections::HashSet, error::Error, hash::Hash};

use anyhow::{anyhow, Result};
use log::{debug, error, info, trace, warn};
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

    // Set key to value in config file, if config file contains it
  fn update_key_code(key: &mut KeyCode, key_name: &str, config_file: &str) {
      let config_name = format!("{KEYCONFIG_PREFIX}.{key_name}");
    let key_from_config = Self::get_config(&config_name, config_file);

      if let Some(new_key) = key_from_config {
      trace!("Updated action {} to new key {:#?}", key_name, new_key);
        *key = new_key;
      }
  }

  pub fn update(&mut self, data: &str) -> Result<()> {
    Self::update_key_code(&mut self.quit, "quit", data);
    Self::update_key_code(&mut self.refresh, "refresh", data);
    Self::update_key_code(&mut self.go_to_bottom, "go-to-bottom", data);
    Self::update_key_code(&mut self.go_to_top, "go-to-top", data);
    Self::update_key_code(&mut self.down, "down", data);
    Self::update_key_code(&mut self.up, "up", data);
    Self::update_key_code(&mut self.page_down, "page-down", data);
    Self::update_key_code(&mut self.page_up, "page-up", data);
    Self::update_key_code(&mut self.delete, "delete", data);
    Self::update_key_code(&mut self.done, "done", data);
    Self::update_key_code(&mut self.start_stop, "start-stop", data);
    Self::update_key_code(&mut self.quick_tag, "quick-tag", data);
    Self::update_key_code(&mut self.select, "select", data);
    Self::update_key_code(&mut self.select_all, "select-all", data);
    Self::update_key_code(&mut self.undo, "undo", data);
    Self::update_key_code(&mut self.edit, "edit", data);
    Self::update_key_code(&mut self.duplicate, "duplicate", data);
    Self::update_key_code(&mut self.modify, "modify", data);
    Self::update_key_code(&mut self.shell, "shell", data);
    Self::update_key_code(&mut self.log, "log", data);
    Self::update_key_code(&mut self.add, "add", data);
    Self::update_key_code(&mut self.annotate, "annotate", data);
    Self::update_key_code(&mut self.filter, "filter", data);
    Self::update_key_code(&mut self.zoom, "zoom", data);
    Self::update_key_code(&mut self.context_menu, "context-menu", data);
    Self::update_key_code(&mut self.next_tab, "next-tab", data);
    Self::update_key_code(&mut self.previous_tab, "previous-tab", data);
    Self::update_key_code(&mut self.shortcut0, "shortcut0", data);
    Self::update_key_code(&mut self.shortcut1, "shortcut1", data);
    Self::update_key_code(&mut self.shortcut2, "shortcut2", data);
    Self::update_key_code(&mut self.shortcut3, "shortcut3", data);
    Self::update_key_code(&mut self.shortcut4, "shortcut4", data);
    Self::update_key_code(&mut self.shortcut5, "shortcut5", data);
    Self::update_key_code(&mut self.shortcut6, "shortcut6", data);
    Self::update_key_code(&mut self.shortcut7, "shortcut7", data);
    Self::update_key_code(&mut self.shortcut8, "shortcut8", data);
    Self::update_key_code(&mut self.shortcut9, "shortcut9", data);

    let keys_to_check = self.keycodes_for_duplicate_check();
    self.check_duplicates(keys_to_check)
  }

  fn keycodes_for_duplicate_check(&self) -> Vec<&KeyCode> {
    vec![
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
    ]
  }

  pub fn check_duplicates(&self, mut elements: Vec<&KeyCode>) -> Result<()> {
    let l = elements.len();
    // TODO: Write Ord implementation for KeyCode.
    // Vecs need to be sorted for dedup to work correctly.
    elements.dedup();
    if l == elements.len() {
      Ok(())
    } else {
      Err(anyhow!("Duplicate keys found in key config"))
    }
  }

  fn get_config(config: &str, data: &str) -> Option<KeyCode> {
    for line in data.split('\n') {
      // Provide leeway for swapped - and _ in keyconfigs
      let config_variants = vec![config.to_owned(), config.replace('-', "_")];

      for config in &config_variants {
        if !line.starts_with(config) {
          continue;
        }

        let trimmed_line = line
          .trim_start_matches(config)
          .trim_start()
          .trim_start_matches('=')
          .trim_end()
          .to_string();

        let chars: Vec<char> = trimmed_line.chars().collect();

        match chars.len() {
          0 => error!("Found no override key for action {} in line {}, only the config prefix", config, line),
          1 => {
            let key_char = chars.first();
            match key_char {
              Some(key_char) => return Some(KeyCode::Char(*key_char)),
              None => error!("Encountered impossible error. Could not fetch first character in Vector of length 1"),
            }
          }
          _ => error!(
            "Found multiple characters({}) in {} for action {}, instead of the expected 1",
            chars.len(),
            line,
            config
          ),
        }
      }
    }

    trace!("Could not find a key override for action {}", config);
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Test if duplicate keys will produce a corresponding error
  #[ignore = "Needs sorting in check_duplicates"]
  #[test]
  fn test_duplicate_key_error() {
    let kc = KeyConfig::default();

    let mut keys_to_check = kc.keycodes_for_duplicate_check();

    // Replace first and last with colliding key
    // This way the duplicate check for non-consecutive keys is assured and correct sorting is tested
    assert!(keys_to_check.len() >= 3);
    *keys_to_check.first_mut().unwrap() = &KeyCode::Char('E');
    *keys_to_check.last_mut().unwrap() = &KeyCode::Char('E');

    let res = kc.check_duplicates(keys_to_check);
    assert!(res.is_err())
  }

  #[test]
  fn test_read_key_config() {
    let config_prefix = "uda.taskwarrior-tui.keyconfig.quit";
    let config_name = format!("{config_prefix}");

    let valid_line = "uda.taskwarrior-tui.keyconfig.quit=q";
    assert!(KeyConfig::get_config(&config_name, valid_line).is_some());

    let invalid_line = "uda.taskwarrior-tui.keyconfig.quit=";
    assert!(KeyConfig::get_config(&config_name, invalid_line).is_none());

    let invalid_line = "uda.taskwarrior-tui.keyconfig.quit=Qt";
    assert!(KeyConfig::get_config(&config_name, invalid_line).is_none());
  }
}
