use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
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
  ExecuteShortcut(usize),
  ExecuteTask(TaskCommand),
  RunShell,
  ShowTaskReport,
  TaskDetailsUpdateUuid(uuid::Uuid),
  TaskDetailsUpdateData((uuid::Uuid, String)),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskCommand {
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

impl<'de> Deserialize<'de> for Action {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct ActionVisitor;

    impl<'de> Visitor<'de> for ActionVisitor {
      type Value = Action;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid string representation of Action")
      }

      fn visit_str<E>(self, value: &str) -> Result<Action, E>
      where
        E: de::Error,
      {
        match value {
          "Tick" => Ok(Action::Tick),
          "Render" => Ok(Action::Render),
          "Suspend" => Ok(Action::Suspend),
          "Resume" => Ok(Action::Resume),
          "Quit" => Ok(Action::Quit),
          "Refresh" => Ok(Action::Refresh),
          "Help" => Ok(Action::Help),
          "MoveDown" => Ok(Action::MoveDown),
          "MoveUp" => Ok(Action::MoveUp),
          "MoveBottom" => Ok(Action::MoveBottom),
          "MoveTop" => Ok(Action::MoveTop),
          "MoveLeft" => Ok(Action::MoveLeft),
          "MoveRight" => Ok(Action::MoveRight),
          "MoveHome" => Ok(Action::MoveHome),
          "MoveEnd" => Ok(Action::MoveEnd),
          "ToggleMark" => Ok(Action::ToggleMark),
          "ToggleMarkAll" => Ok(Action::ToggleMarkAll),
          "Select" => Ok(Action::Select),
          "SelectAll" => Ok(Action::SelectAll),
          "ToggleZoom" => Ok(Action::ToggleZoom),
          "Context" => Ok(Action::Context),
          "RunShell" => Ok(Action::RunShell),
          "ShowTaskReport" => Ok(Action::ShowTaskReport),
          data if data.starts_with("Error(") => {
            let error_msg = data.trim_start_matches("Error(").trim_end_matches(")");
            Ok(Action::Error(error_msg.to_string()))
          },
          data if data.starts_with("Resize(") => {
            let parts: Vec<&str> = data.trim_start_matches("Resize(").trim_end_matches(")").split(',').collect();
            if parts.len() == 2 {
              let width: u16 = parts[0].trim().parse().map_err(E::custom)?;
              let height: u16 = parts[1].trim().parse().map_err(E::custom)?;
              Ok(Action::Resize(width, height))
            } else {
              Err(E::custom(format!("Invalid Resize format: {}", value)))
            }
          },
          data if data.starts_with("ExecuteShortcut(") => {
            let index: usize =
              data.trim_start_matches("ExecuteShortcut(").trim_end_matches(")").parse().map_err(E::custom)?;
            Ok(Action::ExecuteShortcut(index))
          },
          data if data.starts_with("ExecuteTask(") => {
            match data {
              "ExecuteTask(Undo)" => Ok(Action::ExecuteTask(TaskCommand::Undo)),
              "ExecuteTask(Edit)" => Ok(Action::ExecuteTask(TaskCommand::Edit)),
              "ExecuteTask(Tag)" => Ok(Action::ExecuteTask(TaskCommand::Tag)),
              "ExecuteTask(Start)" => Ok(Action::ExecuteTask(TaskCommand::Start)),
              "ExecuteTask(Stop)" => Ok(Action::ExecuteTask(TaskCommand::Stop)),
              "ExecuteTask(Modify)" => Ok(Action::ExecuteTask(TaskCommand::Modify)),
              "ExecuteTask(Log)" => Ok(Action::ExecuteTask(TaskCommand::Log)),
              "ExecuteTask(Annotate)" => Ok(Action::ExecuteTask(TaskCommand::Annotate)),
              "ExecuteTask(Filter)" => Ok(Action::ExecuteTask(TaskCommand::Filter)),
              "ExecuteTask(Add)" => Ok(Action::ExecuteTask(TaskCommand::Add)),
              _ => Err(E::custom(format!("Unknown ExecuteTask variant: {}", value))),
            }
          },
          _ => Err(E::custom(format!("Unknown Action variant: {}", value))),
        }
      }
    }

    deserializer.deserialize_str(ActionVisitor)
  }
}
