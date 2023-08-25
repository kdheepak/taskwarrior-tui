#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum Action {
  Report,
  Filter,
  Add,
  Annotate,
  Subprocess,
  Log,
  Modify,
  HelpPopup,
  ContextMenu,
  Jump,
  DeletePrompt,
  UndoPrompt,
  DonePrompt,
  Error,
}
