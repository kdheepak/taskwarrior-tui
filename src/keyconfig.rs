use std::{collections::HashSet, error::Error, hash::Hash};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::event::KeyCode;

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
    let quit = Self::get_config("uda.taskwarrior-tui.keyconfig.quit", data);
    let refresh = Self::get_config("uda.taskwarrior-tui.keyconfig.refresh", data);
    let go_to_bottom = Self::get_config("uda.taskwarrior-tui.keyconfig.go-to-bottom", data);
    let go_to_top = Self::get_config("uda.taskwarrior-tui.keyconfig.go-to-top", data);
    let down = Self::get_config("uda.taskwarrior-tui.keyconfig.down", data);
    let up = Self::get_config("uda.taskwarrior-tui.keyconfig.up", data);
    let page_down = Self::get_config("uda.taskwarrior-tui.keyconfig.page-down", data);
    let page_up = Self::get_config("uda.taskwarrior-tui.keyconfig.page-up", data);
    let delete = Self::get_config("uda.taskwarrior-tui.keyconfig.delete", data);
    let done = Self::get_config("uda.taskwarrior-tui.keyconfig.done", data);
    let start_stop = Self::get_config("uda.taskwarrior-tui.keyconfig.start-stop", data);
    let quick_tag = Self::get_config("uda.taskwarrior-tui.keyconfig.quick-tag", data);
    let select = Self::get_config("uda.taskwarrior-tui.keyconfig.select", data);
    let select_all = Self::get_config("uda.taskwarrior-tui.keyconfig.select-all", data);
    let undo = Self::get_config("uda.taskwarrior-tui.keyconfig.undo", data);
    let edit = Self::get_config("uda.taskwarrior-tui.keyconfig.edit", data);
    let modify = Self::get_config("uda.taskwarrior-tui.keyconfig.modify", data);
    let shell = Self::get_config("uda.taskwarrior-tui.keyconfig.shell", data);
    let log = Self::get_config("uda.taskwarrior-tui.keyconfig.log", data);
    let add = Self::get_config("uda.taskwarrior-tui.keyconfig.add", data);
    let annotate = Self::get_config("uda.taskwarrior-tui.keyconfig.annotate", data);
    let filter = Self::get_config("uda.taskwarrior-tui.keyconfig.filter", data);
    let zoom = Self::get_config("uda.taskwarrior-tui.keyconfig.zoom", data);
    let context_menu = Self::get_config("uda.taskwarrior-tui.keyconfig.context-menu", data);
    let next_tab = Self::get_config("uda.taskwarrior-tui.keyconfig.next-tab", data);
    let previous_tab = Self::get_config("uda.taskwarrior-tui.keyconfig.previous-tab", data);

    self.quit = quit.unwrap_or(self.quit);
    self.refresh = refresh.unwrap_or(self.refresh);
    self.go_to_bottom = go_to_bottom.unwrap_or(self.go_to_bottom);
    self.go_to_top = go_to_top.unwrap_or(self.go_to_top);
    self.down = down.unwrap_or(self.down);
    self.up = up.unwrap_or(self.up);
    self.page_down = page_down.unwrap_or(self.page_down);
    self.page_up = page_up.unwrap_or(self.page_up);
    self.delete = delete.unwrap_or(self.delete);
    self.done = done.unwrap_or(self.done);
    self.start_stop = start_stop.unwrap_or(self.start_stop);
    self.quick_tag = quick_tag.unwrap_or(self.quick_tag);
    self.select = select.unwrap_or(self.select);
    self.select_all = select_all.unwrap_or(self.select_all);
    self.undo = undo.unwrap_or(self.undo);
    self.edit = edit.unwrap_or(self.edit);
    self.modify = modify.unwrap_or(self.modify);
    self.shell = shell.unwrap_or(self.shell);
    self.log = log.unwrap_or(self.log);
    self.add = add.unwrap_or(self.add);
    self.annotate = annotate.unwrap_or(self.annotate);
    self.filter = filter.unwrap_or(self.filter);
    self.zoom = zoom.unwrap_or(self.zoom);
    self.context_menu = context_menu.unwrap_or(self.context_menu);
    self.next_tab = next_tab.unwrap_or(self.next_tab);
    self.previous_tab = previous_tab.unwrap_or(self.previous_tab);

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
        if line.len() == 1 {
          return Some(KeyCode::Char(line.chars().next().unwrap()));
        }
      } else if line.starts_with(&config.replace('-', "_")) {
        let line = line.trim_start_matches(&config.replace('-', "_")).trim_start().trim_end().to_string();
        if line.len() == 1 {
          return Some(KeyCode::Char(line.chars().next().unwrap()));
        }
      }
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
}
