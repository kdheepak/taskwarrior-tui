use crate::util::Key;
use anyhow::{anyhow, Result};
use async_std::process::Command;
use async_std::task;
use futures::join;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyConfig {
    pub quit: Key,
    pub refresh: Key,
    pub go_to_bottom: Key,
    pub go_to_top: Key,
    pub down: Key,
    pub up: Key,
    pub page_down: Key,
    pub page_up: Key,
    pub delete: Key,
    pub done: Key,
    pub start_stop: Key,
    pub select: Key,
    pub select_all: Key,
    pub undo: Key,
    pub edit: Key,
    pub modify: Key,
    pub shell: Key,
    pub log: Key,
    pub add: Key,
    pub annotate: Key,
    pub help: Key,
    pub filter: Key,
    pub zoom: Key,
    pub context_menu: Key,
    pub next_tab: Key,
    pub previous_tab: Key,
    pub shortcut0: Key,
    pub shortcut1: Key,
    pub shortcut2: Key,
    pub shortcut3: Key,
    pub shortcut4: Key,
    pub shortcut5: Key,
    pub shortcut6: Key,
    pub shortcut7: Key,
    pub shortcut8: Key,
    pub shortcut9: Key,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            quit: Key::Char('q'),
            refresh: Key::Char('r'),
            go_to_bottom: Key::Char('G'),
            go_to_top: Key::Char('g'),
            down: Key::Char('j'),
            up: Key::Char('k'),
            page_down: Key::Char('J'),
            page_up: Key::Char('K'),
            delete: Key::Char('x'),
            done: Key::Char('d'),
            start_stop: Key::Char('s'),
            select: Key::Char('v'),
            select_all: Key::Char('V'),
            undo: Key::Char('u'),
            edit: Key::Char('e'),
            modify: Key::Char('m'),
            shell: Key::Char('!'),
            log: Key::Char('l'),
            add: Key::Char('a'),
            annotate: Key::Char('A'),
            help: Key::Char('?'),
            filter: Key::Char('/'),
            zoom: Key::Char('z'),
            context_menu: Key::Char('c'),
            next_tab: Key::Char(']'),
            previous_tab: Key::Char('['),
            shortcut0: Key::Char('0'),
            shortcut1: Key::Char('1'),
            shortcut2: Key::Char('2'),
            shortcut3: Key::Char('3'),
            shortcut4: Key::Char('4'),
            shortcut5: Key::Char('5'),
            shortcut6: Key::Char('6'),
            shortcut7: Key::Char('7'),
            shortcut8: Key::Char('8'),
            shortcut9: Key::Char('9'),
        }
    }
}

impl KeyConfig {
    pub async fn new(data: &str) -> Result<Self> {
        let mut kc = Self::default();
        kc.update(data).await?;
        Ok(kc)
    }

    pub async fn update(&mut self, data: &str) -> Result<()> {
        let quit = self.get_config("uda.taskwarrior-tui.keyconfig.quit", data);
        let refresh = self.get_config("uda.taskwarrior-tui.keyconfig.refresh", data);
        let go_to_bottom = self.get_config("uda.taskwarrior-tui.keyconfig.go-to-bottom", data);
        let go_to_top = self.get_config("uda.taskwarrior-tui.keyconfig.go-to-top", data);
        let down = self.get_config("uda.taskwarrior-tui.keyconfig.down", data);
        let up = self.get_config("uda.taskwarrior-tui.keyconfig.up", data);
        let page_down = self.get_config("uda.taskwarrior-tui.keyconfig.page-down", data);
        let page_up = self.get_config("uda.taskwarrior-tui.keyconfig.page-up", data);
        let delete = self.get_config("uda.taskwarrior-tui.keyconfig.delete", data);
        let done = self.get_config("uda.taskwarrior-tui.keyconfig.done", data);
        let start_stop = self.get_config("uda.taskwarrior-tui.keyconfig.start-stop", data);
        let select = self.get_config("uda.taskwarrior-tui.keyconfig.select", data);
        let select_all = self.get_config("uda.taskwarrior-tui.keyconfig.select-all", data);
        let undo = self.get_config("uda.taskwarrior-tui.keyconfig.undo", data);
        let edit = self.get_config("uda.taskwarrior-tui.keyconfig.edit", data);
        let modify = self.get_config("uda.taskwarrior-tui.keyconfig.modify", data);
        let shell = self.get_config("uda.taskwarrior-tui.keyconfig.shell", data);
        let log = self.get_config("uda.taskwarrior-tui.keyconfig.log", data);
        let add = self.get_config("uda.taskwarrior-tui.keyconfig.add", data);
        let annotate = self.get_config("uda.taskwarrior-tui.keyconfig.annotate", data);
        let filter = self.get_config("uda.taskwarrior-tui.keyconfig.filter", data);
        let zoom = self.get_config("uda.taskwarrior-tui.keyconfig.zoom", data);
        let context_menu = self.get_config("uda.taskwarrior-tui.keyconfig.context-menu", data);
        let next_tab = self.get_config("uda.taskwarrior-tui.keyconfig.next-tab", data);
        let previous_tab = self.get_config("uda.taskwarrior-tui.keyconfig.previous-tab", data);

        let (
            quit,
            refresh,
            go_to_bottom,
            go_to_top,
            down,
            up,
            page_down,
            page_up,
            delete,
            done,
            start_stop,
            select,
            select_all,
            undo,
            edit,
            modify,
            shell,
            log,
            add,
            annotate,
            filter,
            zoom,
            context_menu,
            next_tab,
            previous_tab,
        ) = join!(
            quit,
            refresh,
            go_to_bottom,
            go_to_top,
            down,
            up,
            page_down,
            page_up,
            delete,
            done,
            start_stop,
            select,
            select_all,
            undo,
            edit,
            modify,
            shell,
            log,
            add,
            annotate,
            filter,
            zoom,
            context_menu,
            next_tab,
            previous_tab,
        );

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

    async fn get_config(&self, config: &str, data: &str) -> Option<Key> {
        for line in data.split('\n') {
            if line.starts_with(config) {
                let line = line.trim_start_matches(config).trim_start().trim_end().to_string();
                if line.len() == 1 {
                    return Some(Key::Char(line.chars().next().unwrap()));
                } else {
                    return None;
                }
            } else if line.starts_with(&config.replace('-', "_")) {
                let line = line
                    .trim_start_matches(&config.replace('-', "_"))
                    .trim_start()
                    .trim_end()
                    .to_string();
                if line.len() == 1 {
                    return Some(Key::Char(line.chars().next().unwrap()));
                } else {
                    return None;
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
