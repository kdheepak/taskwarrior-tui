use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MediaKeyCode, ModifierKeyCode};

fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
  let raw_lower = raw.to_ascii_lowercase();
  let (remaining, modifiers) = extract_modifiers(&raw_lower);
  parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
  let mut modifiers = KeyModifiers::empty();
  let mut current = raw;

  loop {
    match current {
      rest if rest.starts_with("ctrl-") => {
        modifiers.insert(KeyModifiers::CONTROL);
        current = &rest[5..];
      }
      rest if rest.starts_with("alt-") => {
        modifiers.insert(KeyModifiers::ALT);
        current = &rest[4..];
      }
      rest if rest.starts_with("shift-") => {
        modifiers.insert(KeyModifiers::SHIFT);
        current = &rest[6..];
      }
      _ => break, // break out of the loop if no known prefix is detected
    };
  }

  (current, modifiers)
}

fn parse_key_code_with_modifiers(raw: &str, mut modifiers: KeyModifiers) -> Result<KeyEvent, String> {
  let c = match raw {
    "esc" => KeyCode::Esc,
    "enter" => KeyCode::Enter,
    "left" => KeyCode::Left,
    "right" => KeyCode::Right,
    "up" => KeyCode::Up,
    "down" => KeyCode::Down,
    "home" => KeyCode::Home,
    "end" => KeyCode::End,
    "pageup" => KeyCode::PageUp,
    "pagedown" => KeyCode::PageDown,
    "backtab" => {
      modifiers.insert(KeyModifiers::SHIFT);
      KeyCode::BackTab
    }
    "backspace" => KeyCode::Backspace,
    "delete" => KeyCode::Delete,
    "insert" => KeyCode::Insert,
    "f1" => KeyCode::F(1),
    "f2" => KeyCode::F(2),
    "f3" => KeyCode::F(3),
    "f4" => KeyCode::F(4),
    "f5" => KeyCode::F(5),
    "f6" => KeyCode::F(6),
    "f7" => KeyCode::F(7),
    "f8" => KeyCode::F(8),
    "f9" => KeyCode::F(9),
    "f10" => KeyCode::F(10),
    "f11" => KeyCode::F(11),
    "f12" => KeyCode::F(12),
    "space" => KeyCode::Char(' '),
    "tab" => KeyCode::Tab,
    c if c.len() == 1 => {
      let mut c = c.chars().next().unwrap();
      if modifiers.contains(KeyModifiers::SHIFT) {
        c = c.to_ascii_uppercase();
      }
      KeyCode::Char(c)
    }
    _ => return Err(format!("Unable to parse {raw}")),
  };
  Ok(KeyEvent::new(c, modifiers))
}

pub fn parse_key_sequence(raw: &str) -> Result<Vec<KeyEvent>, String> {
  if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
    return Err(format!("Unable to parse `{}`", raw));
  }
  let raw = if !raw.contains("><") {
    let raw = raw.strip_prefix("<").unwrap_or(raw);
    let raw = raw.strip_prefix(">").unwrap_or(raw);
    raw
  } else {
    raw
  };
  let sequences = raw
    .split("><")
    .map(|seq| {
      if seq.starts_with('<') {
        &seq[1..]
      } else if seq.ends_with('>') {
        &seq[..seq.len() - 1]
      } else {
        seq
      }
    })
    .collect::<Vec<_>>();

  sequences.into_iter().map(parse_key_event).collect()
}

pub fn key_event_to_string(event: KeyEvent) -> String {
  let mut result = String::new();

  result.push('<');

  // Add modifiers
  if event.modifiers.contains(KeyModifiers::CONTROL) {
    result.push_str("ctrl-");
  }
  if event.modifiers.contains(KeyModifiers::ALT) {
    result.push_str("alt-");
  }
  if event.modifiers.contains(KeyModifiers::SHIFT) {
    result.push_str("shift-");
  }

  match event.code {
    KeyCode::Char(' ') => result.push_str("space"),
    KeyCode::Char(c) => result.push(c),
    KeyCode::Enter => result.push_str("enter"),
    KeyCode::Esc => result.push_str("esc"),
    KeyCode::Left => result.push_str("left"),
    KeyCode::Right => result.push_str("right"),
    KeyCode::Up => result.push_str("up"),
    KeyCode::Down => result.push_str("down"),
    KeyCode::Home => result.push_str("home"),
    KeyCode::End => result.push_str("end"),
    KeyCode::PageUp => result.push_str("pageup"),
    KeyCode::PageDown => result.push_str("pagedown"),
    KeyCode::BackTab => result.push_str("backtab"),
    KeyCode::Delete => result.push_str("delete"),
    KeyCode::Insert => result.push_str("insert"),
    KeyCode::F(n) => result.push_str(&format!("f{}", n)),
    KeyCode::Backspace => result.push_str("backspace"),
    KeyCode::Tab => result.push_str("tab"),
    KeyCode::Null => result.push_str("null"),
    KeyCode::CapsLock => result.push_str("capslock"),
    KeyCode::ScrollLock => result.push_str("scrolllock"),
    KeyCode::NumLock => result.push_str("numlock"),
    KeyCode::PrintScreen => result.push_str("printscreen"),
    KeyCode::Pause => result.push_str("pause"),
    KeyCode::Menu => result.push_str("menu"),
    KeyCode::KeypadBegin => result.push_str("keypadbegin"),
    KeyCode::Media(media) => match media {
      MediaKeyCode::Play => result.push_str("play"),
      MediaKeyCode::Pause => result.push_str("pause"),
      MediaKeyCode::PlayPause => result.push_str("playpause"),
      MediaKeyCode::Reverse => result.push_str("reverse"),
      MediaKeyCode::Stop => result.push_str("stop"),
      MediaKeyCode::FastForward => result.push_str("fastforward"),
      MediaKeyCode::Rewind => result.push_str("rewind"),
      MediaKeyCode::TrackNext => result.push_str("tracknext"),
      MediaKeyCode::TrackPrevious => result.push_str("trackprevious"),
      MediaKeyCode::Record => result.push_str("record"),
      MediaKeyCode::LowerVolume => result.push_str("lowervolume"),
      MediaKeyCode::RaiseVolume => result.push_str("raisevolume"),
      MediaKeyCode::MuteVolume => result.push_str("mutevolume"),
    },
    KeyCode::Modifier(keycode) => match keycode {
      ModifierKeyCode::LeftShift => result.push_str("leftshift"),
      ModifierKeyCode::LeftControl => result.push_str("leftcontrol"),
      ModifierKeyCode::LeftAlt => result.push_str("leftalt"),
      ModifierKeyCode::LeftSuper => result.push_str("leftsuper"),
      ModifierKeyCode::LeftHyper => result.push_str("lefthyper"),
      ModifierKeyCode::LeftMeta => result.push_str("leftmeta"),
      ModifierKeyCode::RightShift => result.push_str("rightshift"),
      ModifierKeyCode::RightControl => result.push_str("rightcontrol"),
      ModifierKeyCode::RightAlt => result.push_str("rightalt"),
      ModifierKeyCode::RightSuper => result.push_str("rightsuper"),
      ModifierKeyCode::RightHyper => result.push_str("righthyper"),
      ModifierKeyCode::RightMeta => result.push_str("rightmeta"),
      ModifierKeyCode::IsoLevel3Shift => result.push_str("isolevel3shift"),
      ModifierKeyCode::IsoLevel5Shift => result.push_str("isolevel5shift"),
    },
  }

  result.push('>');

  result
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  fn test_event_to_string() {
    let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    println!("{}", key_event_to_string(event)); // Outputs: ctrl-a
  }

  #[test]
  fn test_single_key_sequence() {
    let result = parse_key_sequence("a");
    assert_eq!(result.unwrap(), vec![KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())]);

    let result = parse_key_sequence("<a><b>");
    assert_eq!(
      result.unwrap(),
      vec![
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
        KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty())
      ]
    );

    let result = parse_key_sequence("<Ctrl-a><Alt-b>");
    assert_eq!(
      result.unwrap(),
      vec![
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
        KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT)
      ]
    );
    let result = parse_key_sequence("<Ctrl-a>");
    assert_eq!(result.unwrap(), vec![KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),]);
    let result = parse_key_sequence("<Ctrl-Alt-a>");
    assert_eq!(
      result.unwrap(),
      vec![KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL | KeyModifiers::ALT),]
    );
    assert!(parse_key_sequence("Ctrl-a>").is_err());
    assert!(parse_key_sequence("<Ctrl-a").is_err());
  }

  #[test]
  fn test_simple_keys() {
    assert_eq!(parse_key_event("a").unwrap(), KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));

    assert_eq!(parse_key_event("enter").unwrap(), KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));

    assert_eq!(parse_key_event("esc").unwrap(), KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
  }

  #[test]
  fn test_with_modifiers() {
    assert_eq!(
      parse_key_event("ctrl-a").unwrap(),
      KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
    );

    assert_eq!(parse_key_event("alt-enter").unwrap(), KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));

    assert_eq!(parse_key_event("shift-esc").unwrap(), KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT));
  }

  #[test]
  fn test_multiple_modifiers() {
    assert_eq!(
      parse_key_event("ctrl-alt-a").unwrap(),
      KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL | KeyModifiers::ALT)
    );

    assert_eq!(
      parse_key_event("ctrl-shift-enter").unwrap(),
      KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
    );
  }

  #[test]
  fn test_invalid_keys() {
    assert!(parse_key_event("invalid-key").is_err());
    assert!(parse_key_event("ctrl-invalid-key").is_err());
  }

  #[test]
  fn test_case_insensitivity() {
    assert_eq!(
      parse_key_event("CTRL-a").unwrap(),
      KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
    );

    assert_eq!(parse_key_event("AlT-eNtEr").unwrap(), KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
  }
}
