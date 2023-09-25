use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
  Tick,
  Render,
  Resize(u16, u16),
  Suspend,
  Resume,
  Quit,
  Refresh,
  Error(String),
  Help,
  MoveDown,
  MoveUp,
  MoveBottom,
  MoveTop,
  MoveLeft,
  MoveRight,
  MoveHome,
  MoveEnd,
  ToggleMark,
  ToggleMarkAll,
  Select,
  SelectAll,
  ToggleZoom,
  Context,
  RunShortcut(usize),
  RunShell,
  Task,
  ShowTaskReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Task {
  Undo,
  Edit,
  Tag,
  Start,
  Stop,
  Modify,
  Log,
  Annotate,
  Filter,
  Add,
}
