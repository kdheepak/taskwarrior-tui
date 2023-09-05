use std::ops::{Deref, DerefMut};
use std::{collections::HashMap, error::Error, str};

use color_eyre::eyre::{eyre, Context, Result};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{self, Serialize, SerializeMap, Serializer};

use crate::keyevent::key_event_to_string;
use crate::{action::Action, keyevent::parse_key_sequence};

#[derive(Clone, Debug, Default)]
pub struct KeyMap(pub std::collections::HashMap<Vec<KeyEvent>, Action>);

impl Deref for KeyMap {
  type Target = std::collections::HashMap<Vec<KeyEvent>, Action>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for KeyMap {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl KeyMap {
  pub fn validate(&self) -> Result<(), String> {
    let mut sorted_sequences: Vec<_> = self.keys().collect();
    sorted_sequences.sort_by_key(|seq| seq.len());

    for i in 0..sorted_sequences.len() {
      for j in i + 1..sorted_sequences.len() {
        if sorted_sequences[j].starts_with(sorted_sequences[i]) {
          return Err(format!(
            "Conflict detected: Sequence {:?} is a prefix of sequence {:?}",
            sorted_sequences[i], sorted_sequences[j]
          ));
        }
      }
    }

    Ok(())
  }
}

impl Serialize for KeyMap {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    // Begin serializing a map.
    let mut map = serializer.serialize_map(Some(self.0.len()))?;

    for (key_sequence, action) in &self.0 {
      let key_string = key_sequence
        .iter()
        .map(|key_event| key_event_to_string(*key_event))
        .collect::<Vec<_>>()
        .join("");

      map.serialize_entry(&key_string, action)?;
    }

    // End serialization.
    map.end()
  }
}

impl<'de> Deserialize<'de> for KeyMap {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct KeyMapVisitor;

    impl<'de> Visitor<'de> for KeyMapVisitor {
      type Value = KeyMap;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a keymap with string representation of KeyEvent as key and Action as value")
      }

      fn visit_map<M>(self, mut access: M) -> Result<KeyMap, M::Error>
      where
        M: MapAccess<'de>,
      {
        let mut keymap = std::collections::HashMap::new();

        // While there are entries in the map, read them
        while let Some((key_sequence_str, action)) = access.next_entry::<String, Action>()? {
          let key_sequence = parse_key_sequence(&key_sequence_str).map_err(de::Error::custom)?;

          if let Some(old_action) = keymap.insert(key_sequence, action.clone()) {
            if old_action != action {
              return Err(format!("Found a {:?} for both {:?} and {:?}", key_sequence_str, old_action, action)).map_err(de::Error::custom);
            }
          }
        }

        Ok(KeyMap(keymap))
      }
    }
    deserializer.deserialize_map(KeyMapVisitor)
  }
}

#[cfg(test)]
mod validate_tests {
  use super::*;
  use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

  #[test]
  fn test_no_conflict() {
    let mut map = std::collections::HashMap::new();
    map.insert(vec![KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())], Action::Quit);
    map.insert(vec![KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty())], Action::Quit);
    let keymap = KeyMap(map);

    assert!(keymap.validate().is_ok());
  }

  #[test]
  fn test_conflict_prefix() {
    let mut map = std::collections::HashMap::new();
    map.insert(vec![KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())], Action::Quit);
    map.insert(
      vec![
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()),
      ],
      Action::Quit,
    );
    let keymap = KeyMap(map);

    assert!(keymap.validate().is_err());
  }

  #[test]
  fn test_no_conflict_different_modifiers() {
    let mut map = std::collections::HashMap::new();
    map.insert(vec![KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)], Action::Quit);
    map.insert(vec![KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT)], Action::Quit);
    let keymap = KeyMap(map);

    assert!(keymap.validate().is_ok());
  }

  #[test]
  fn test_no_conflict_multiple_keys() {
    let mut map = std::collections::HashMap::new();
    map.insert(
      vec![
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()),
      ],
      Action::Quit,
    );
    map.insert(
      vec![
        KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
      ],
      Action::Quit,
    );
    let keymap = KeyMap(map);

    assert!(keymap.validate().is_ok());
  }

  #[test]
  fn test_conflict_three_keys() {
    let mut map = std::collections::HashMap::new();
    map.insert(
      vec![
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty()),
      ],
      Action::Quit,
    );
    map.insert(
      vec![
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()),
      ],
      Action::Quit,
    );
    let keymap = KeyMap(map);

    assert!(keymap.validate().is_err());
  }
}
