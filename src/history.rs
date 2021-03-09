use std::{
    fs::{create_dir_all, write, File},
    io::{BufRead, BufReader, Error, Read},
    path::{Path, PathBuf},
};
use itertools::Itertools;


#[derive(Clone)]
pub struct Commands {
    pub all: Vec<String>,
}

impl Commands {
    pub fn from_history(shell: &str, history: &[String]) -> Self {
        Self {
            all: history.to_vec().into_iter().filter(|s| s.starts_with("task")).unique().collect(),
        }
    }
}

pub struct History {
    pub history: Vec<String>,
    pub search: String,
    pub shell: String,
}

impl History {
    pub fn new(shell: String) -> Self {
        let (history, commands) = match shell.as_str() {
            "bash" => get_bash_history(),
            "zsh" => get_zsh_history(),
            _ => unreachable!(),
        };
        Self {
            history: commands.all,
            search: String::new(),
            shell,
        }
    }
}

pub fn get_bash_history() -> (Vec<String>, Commands) {
    let history = read_from_home(".bash_history").unwrap();
    let commands = Commands::from_history("bash", &history);
    (history, commands)
}

pub fn get_zsh_history() -> (Vec<String>, Commands) {
    let history = zsh::process_history()
        .split('\n')
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let commands = Commands::from_history("zsh", &history);
    (history, commands)
}

pub fn read_from_home(path: impl AsRef<Path>) -> Result<Vec<String>, Error> {
    /* `path` is relative to home directory */
    let home = dirs::home_dir().unwrap();
    let target = home.join(path);
    if target.exists() {
        read_file(target)
    } else {
        Ok(Vec::new())
    }
}

fn read_file(target: PathBuf) -> Result<Vec<String>, Error> {
    let file = File::open(target)?;
    let reader = BufReader::new(file);
    reader.lines().collect::<Result<Vec<_>, _>>()
}


pub mod zsh {
    use super::*;
    use regex::Regex;

    pub fn read_as_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>, Error> {
        let home = dirs::home_dir().unwrap();
        let target = home.join(path);
        let mut file = File::open(target)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn process_history() -> String {
        let history = read_as_bytes(".zsh_history").unwrap();
        let unmetafied = unmetafy(history);
        remove_timestamps(String::from_utf8(unmetafied).unwrap())
    }

    fn unmetafy(mut bytestring: Vec<u8>) -> Vec<u8> {
        /* Unmetafying zsh history requires looping over the bytestring, removing
         * each encountered Meta character, and XOR-ing the following byte with 32.
         *
         * For instance:
         *
         * Input: ('a', 'b', 'c', Meta, 'd', 'e', 'f')
         * Wanted: ('a', 'b', 'c', 'd' ^ 32, 'e', 'f')
         */
        const ZSH_META: u8 = 0x83;
        for index in (0..bytestring.len()).rev() {
            if bytestring[index] == ZSH_META {
                bytestring.remove(index);
                bytestring[index] ^= 32;
            }
        }
        bytestring
    }

    fn remove_timestamps(history: String) -> String {
        /* The preceding metadata needs to be stripped
         * because zsh history entries look like below:
         *
         * `: 1330648651:0;sudo reboot`
         */
        let r = Regex::new(r"^: \d{10}:\d;").unwrap();
        history.lines().map(|x| r.replace(x, "") + "\n").collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zsh_history() {
        let h = History::new("zsh".to_string());
        dbg!(h.history);
    }
}
